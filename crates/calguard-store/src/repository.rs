use crate::db::Database;
use crate::migrations::migrate;
use calguard_core::{
    AnalysisSettings, CalendarSource, CalendarSourceKind, Event, EventStatus, RawCalendar,
    SyncStatus, Transparency,
};
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct Repository {
    database: Database,
}

#[derive(Default)]
pub struct SourceUpdate<'a> {
    pub name: Option<&'a str>,
    pub enabled: Option<bool>,
    pub color: Option<&'a str>,
    pub status: Option<SyncStatus>,
    pub error_message: Option<Option<&'a str>>,
    pub last_synced_at: Option<Option<DateTime<Utc>>>,
}

impl Repository {
    pub fn new(database: Database) -> rusqlite::Result<Self> {
        migrate(database.connection())?;
        Ok(Self { database })
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn add_source(&self, mut source: CalendarSource) -> rusqlite::Result<CalendarSource> {
        if source.id.is_empty() {
            source.id = Uuid::new_v4().to_string();
        }
        let (kind, config_json) = kind_to_storage(&source.kind)?;
        self.database.connection().execute(
            r#"
            INSERT INTO calendar_sources (
                id, name, kind, config_json, color, enabled, sync_status, last_error,
                created_at, updated_at, last_synced_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                &source.id,
                &source.name,
                kind,
                &config_json,
                source.color.as_deref(),
                source.enabled as i32,
                sync_status_to_str(source.sync_status),
                source.error_message.as_deref(),
                source.created_at.to_rfc3339(),
                source.updated_at.to_rfc3339(),
                source.last_synced_at.map(|date| date.to_rfc3339()),
            ],
        )?;
        self.get_source(&source.id)
            .map(|source| source.expect("inserted source must exist"))
    }

    pub fn list_sources(&self) -> rusqlite::Result<Vec<CalendarSource>> {
        let mut statement = self.database.connection().prepare(
            r#"
            SELECT id, name, kind, config_json, color, enabled, sync_status, last_error,
                   created_at, updated_at, last_synced_at
            FROM calendar_sources
            ORDER BY created_at ASC
            "#,
        )?;
        let rows = statement.query_map([], row_to_source)?;
        rows.collect()
    }

    pub fn get_source(&self, source_id: &str) -> rusqlite::Result<Option<CalendarSource>> {
        self.database
            .connection()
            .query_row(
                r#"
                SELECT id, name, kind, config_json, color, enabled, sync_status, last_error,
                       created_at, updated_at, last_synced_at
                FROM calendar_sources
                WHERE id = ?1
                "#,
                params![source_id],
                row_to_source,
            )
            .optional()
    }

    pub fn update_source(
        &self,
        source_id: &str,
        update: SourceUpdate<'_>,
    ) -> rusqlite::Result<Option<CalendarSource>> {
        let Some(mut source) = self.get_source(source_id)? else {
            return Ok(None);
        };
        if let Some(name) = update.name {
            source.name = name.to_string();
        }
        if let Some(enabled) = update.enabled {
            source.enabled = enabled;
        }
        if let Some(color) = update.color {
            source.color = Some(color.to_string());
        }
        if let Some(status) = update.status {
            source.sync_status = status;
        }
        if let Some(error_message) = update.error_message {
            source.error_message = error_message.map(ToString::to_string);
        }
        if let Some(last_synced_at) = update.last_synced_at {
            source.last_synced_at = last_synced_at;
        }
        source.updated_at = Utc::now();
        self.database.connection().execute(
            r#"
            UPDATE calendar_sources
            SET name = ?2, color = ?3, enabled = ?4, sync_status = ?5, last_error = ?6,
                updated_at = ?7, last_synced_at = ?8
            WHERE id = ?1
            "#,
            params![
                &source.id,
                &source.name,
                source.color.as_deref(),
                source.enabled as i32,
                sync_status_to_str(source.sync_status),
                source.error_message.as_deref(),
                source.updated_at.to_rfc3339(),
                source.last_synced_at.map(|date| date.to_rfc3339()),
            ],
        )?;
        self.get_source(source_id)
    }

    pub fn remove_source(&self, source_id: &str) -> rusqlite::Result<()> {
        self.database.connection().execute(
            "DELETE FROM calendar_sources WHERE id = ?1",
            params![source_id],
        )?;
        Ok(())
    }

    pub fn store_raw_calendar(
        &self,
        source_id: &str,
        raw_ics: &str,
    ) -> rusqlite::Result<RawCalendar> {
        let raw = RawCalendar {
            id: Uuid::new_v4().to_string(),
            source_id: source_id.to_string(),
            raw_ics: raw_ics.to_string(),
            content_hash: content_hash(raw_ics),
            imported_at: Utc::now(),
        };
        self.database.connection().execute(
            r#"
            INSERT INTO raw_calendars (id, source_id, raw_ics, content_hash, imported_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                &raw.id,
                &raw.source_id,
                &raw.raw_ics,
                &raw.content_hash,
                raw.imported_at.to_rfc3339(),
            ],
        )?;
        Ok(raw)
    }

    pub fn latest_raw_calendar(&self, source_id: &str) -> rusqlite::Result<Option<RawCalendar>> {
        self.database
            .connection()
            .query_row(
                r#"
                SELECT id, source_id, raw_ics, content_hash, imported_at
                FROM raw_calendars
                WHERE source_id = ?1
                ORDER BY imported_at DESC
                LIMIT 1
                "#,
                params![source_id],
                |row| {
                    Ok(RawCalendar {
                        id: row.get(0)?,
                        source_id: row.get(1)?,
                        raw_ics: row.get(2)?,
                        content_hash: row.get(3)?,
                        imported_at: parse_rfc3339(row.get::<_, String>(4)?)?,
                    })
                },
            )
            .optional()
    }

    pub fn replace_source_events(
        &self,
        source_id: &str,
        raw_calendar_id: &str,
        events: &[Event],
    ) -> rusqlite::Result<()> {
        self.database.connection().execute(
            "DELETE FROM events WHERE source_id = ?1",
            params![source_id],
        )?;
        for event in events {
            self.database.connection().execute(
                r#"
                INSERT INTO events (
                    id, source_id, uid, title, description, location, starts_at, ends_at,
                    timezone, is_all_day, status, transparency, recurrence_rule,
                    excluded_dates_json, recurrence_id, raw_calendar_id
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                "#,
                params![
                    &event.id,
                    &event.source_id,
                    &event.uid,
                    &event.title,
                    event.description.as_deref(),
                    event.location.as_deref(),
                    event.starts_at.to_rfc3339(),
                    event.ends_at.to_rfc3339(),
                    event.timezone.as_deref(),
                    event.is_all_day as i32,
                    event_status_to_str(event.status),
                    transparency_to_str(event.transparency),
                    event.recurrence_rule.as_deref(),
                    serde_json::to_string(
                        &event
                            .excluded_dates
                            .iter()
                            .map(DateTime::<Utc>::to_rfc3339)
                            .collect::<Vec<_>>()
                    )
                    .map_err(to_sql_error)?,
                    event.recurrence_id.map(|date| date.to_rfc3339()),
                    raw_calendar_id,
                ],
            )?;
        }
        Ok(())
    }

    pub fn list_events(&self, source_ids: &[String]) -> rusqlite::Result<Vec<Event>> {
        let mut statement = self.database.connection().prepare(
            r#"
            SELECT events.id, events.source_id, uid, title, description, location, starts_at, ends_at,
                   timezone, is_all_day, status, transparency, recurrence_rule, excluded_dates_json,
                   recurrence_id
            FROM events
            JOIN calendar_sources ON calendar_sources.id = events.source_id
            WHERE calendar_sources.enabled = 1
            ORDER BY starts_at ASC
            "#,
        )?;
        let rows = statement.query_map([], row_to_event)?;
        let mut events = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        if !source_ids.is_empty() {
            events.retain(|event| source_ids.contains(&event.source_id));
        }
        Ok(events)
    }

    pub fn get_settings(&self) -> rusqlite::Result<AnalysisSettings> {
        let value = self
            .database
            .connection()
            .query_row(
                "SELECT value_json FROM settings WHERE key = 'analysis_settings'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        match value {
            Some(value) => serde_json::from_str(&value).map_err(to_sql_error),
            None => Ok(AnalysisSettings::default()),
        }
    }

    pub fn update_settings(&self, settings: &AnalysisSettings) -> rusqlite::Result<()> {
        self.database.connection().execute(
            r#"
            INSERT INTO settings (key, value_json, updated_at)
            VALUES ('analysis_settings', ?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at
            "#,
            params![
                serde_json::to_string(settings).map_err(to_sql_error)?,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_ignored_hashes(&self, kind: &str) -> rusqlite::Result<Vec<String>> {
        let mut statement = self
            .database
            .connection()
            .prepare("SELECT target_hash FROM ignored_items WHERE kind = ?1")?;
        let rows = statement.query_map(params![kind], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    pub fn add_ignored_item(
        &self,
        kind: &str,
        target_hash: &str,
        reason: Option<&str>,
    ) -> rusqlite::Result<()> {
        self.database.connection().execute(
            r#"
            INSERT OR REPLACE INTO ignored_items (id, kind, target_hash, reason, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                content_hash(&format!("{kind}:{target_hash}")),
                kind,
                target_hash,
                reason,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn record_export(
        &self,
        format: &str,
        file_path: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        privacy_mode: bool,
    ) -> rusqlite::Result<()> {
        self.database.connection().execute(
            r#"
            INSERT INTO report_exports (
                id, format, file_path, period_start, period_end, privacy_mode, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                Uuid::new_v4().to_string(),
                format,
                file_path,
                period_start.to_rfc3339(),
                period_end.to_rfc3339(),
                privacy_mode as i32,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn clear_cache(&self) -> rusqlite::Result<()> {
        self.database
            .connection()
            .execute("DELETE FROM raw_calendars", [])?;
        self.database
            .connection()
            .execute("DELETE FROM events", [])?;
        Ok(())
    }
}

fn row_to_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<CalendarSource> {
    let kind: String = row.get(2)?;
    let config_json: String = row.get(3)?;
    Ok(CalendarSource {
        id: row.get(0)?,
        name: row.get(1)?,
        kind: storage_to_kind(&kind, &config_json)?,
        color: row.get(4)?,
        enabled: row.get::<_, i32>(5)? != 0,
        sync_status: str_to_sync_status(row.get::<_, String>(6)?.as_str()),
        error_message: row.get(7)?,
        created_at: parse_rfc3339(row.get::<_, String>(8)?)?,
        updated_at: parse_rfc3339(row.get::<_, String>(9)?)?,
        last_synced_at: row
            .get::<_, Option<String>>(10)?
            .map(parse_rfc3339)
            .transpose()?,
    })
}

fn row_to_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<Event> {
    let excluded_dates_json: String = row.get(13)?;
    let excluded_dates = serde_json::from_str::<Vec<String>>(&excluded_dates_json)
        .map_err(to_sql_error)?
        .into_iter()
        .map(parse_rfc3339)
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Event {
        id: row.get(0)?,
        source_id: row.get(1)?,
        uid: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        location: row.get(5)?,
        starts_at: parse_rfc3339(row.get::<_, String>(6)?)?,
        ends_at: parse_rfc3339(row.get::<_, String>(7)?)?,
        timezone: row.get(8)?,
        is_all_day: row.get::<_, i32>(9)? != 0,
        status: str_to_event_status(row.get::<_, String>(10)?.as_str()),
        transparency: str_to_transparency(row.get::<_, String>(11)?.as_str()),
        recurrence_rule: row.get(12)?,
        excluded_dates,
        recurrence_id: row
            .get::<_, Option<String>>(14)?
            .map(parse_rfc3339)
            .transpose()?,
    })
}

fn kind_to_storage(kind: &CalendarSourceKind) -> rusqlite::Result<(&'static str, String)> {
    match kind {
        CalendarSourceKind::LocalIcsFile { .. } => Ok((
            "local_ics_file",
            serde_json::to_string(kind).map_err(to_sql_error)?,
        )),
        CalendarSourceKind::RemoteIcsUrl { .. } => Ok((
            "remote_ics_url",
            serde_json::to_string(kind).map_err(to_sql_error)?,
        )),
    }
}

fn storage_to_kind(kind: &str, config_json: &str) -> rusqlite::Result<CalendarSourceKind> {
    match serde_json::from_str(config_json) {
        Ok(kind) => Ok(kind),
        Err(_) => match kind {
            "local_ics_file" => Ok(CalendarSourceKind::LocalIcsFile {
                path: String::new(),
            }),
            "remote_ics_url" => Ok(CalendarSourceKind::RemoteIcsUrl { url: String::new() }),
            _ => Err(to_sql_error(format!("unknown source kind {kind}"))),
        },
    }
}

fn sync_status_to_str(status: SyncStatus) -> &'static str {
    match status {
        SyncStatus::Idle => "idle",
        SyncStatus::Syncing => "syncing",
        SyncStatus::Success => "success",
        SyncStatus::Failed => "failed",
    }
}

fn str_to_sync_status(status: &str) -> SyncStatus {
    match status {
        "syncing" => SyncStatus::Syncing,
        "success" => SyncStatus::Success,
        "failed" => SyncStatus::Failed,
        _ => SyncStatus::Idle,
    }
}

fn event_status_to_str(status: EventStatus) -> &'static str {
    match status {
        EventStatus::Confirmed => "CONFIRMED",
        EventStatus::Tentative => "TENTATIVE",
        EventStatus::Cancelled => "CANCELLED",
    }
}

fn str_to_event_status(status: &str) -> EventStatus {
    match status {
        "TENTATIVE" => EventStatus::Tentative,
        "CANCELLED" => EventStatus::Cancelled,
        _ => EventStatus::Confirmed,
    }
}

fn transparency_to_str(transparency: Transparency) -> &'static str {
    match transparency {
        Transparency::Opaque => "OPAQUE",
        Transparency::Transparent => "TRANSPARENT",
    }
}

fn str_to_transparency(transparency: &str) -> Transparency {
    match transparency {
        "TRANSPARENT" => Transparency::Transparent,
        _ => Transparency::Opaque,
    }
}

fn parse_rfc3339(value: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|date| date.with_timezone(&Utc))
        .map_err(to_sql_error)
}

fn content_hash(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest[..16]
        .iter()
        .fold(String::with_capacity(32), |mut output, byte| {
            output.push_str(&format!("{byte:02x}"));
            output
        })
}

fn to_sql_error(error: impl std::fmt::Display) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        error.to_string(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use calguard_core::{parse_ics, CalendarSourceKind};

    #[test]
    fn stores_sources_raw_events_settings_and_ignored_items() {
        let repo = Repository::new(Database::in_memory().unwrap()).unwrap();
        let now = Utc::now();
        let source = repo
            .add_source(CalendarSource {
                id: "source-1".to_string(),
                name: "Work".to_string(),
                kind: CalendarSourceKind::LocalIcsFile {
                    path: "/tmp/work.ics".to_string(),
                },
                color: Some("#2563eb".to_string()),
                enabled: true,
                created_at: now,
                updated_at: now,
                last_synced_at: None,
                sync_status: SyncStatus::Idle,
                error_message: None,
            })
            .unwrap();
        assert_eq!(source.name, "Work");

        let raw_text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:1\nSUMMARY:Planning\nDTSTART:20260601T090000Z\nDTEND:20260601T100000Z\nEND:VEVENT\nEND:VCALENDAR\n";
        let raw = repo.store_raw_calendar(&source.id, raw_text).unwrap();
        let parsed = parse_ics(raw_text, &source.id);
        repo.replace_source_events(&source.id, &raw.id, &parsed.events)
            .unwrap();

        let events = repo.list_events(&[]).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Planning");

        repo.update_source(
            &source.id,
            SourceUpdate {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(repo.list_events(&[]).unwrap().is_empty());
        repo.update_source(
            &source.id,
            SourceUpdate {
                enabled: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(repo.list_events(&[]).unwrap().len(), 1);

        let settings = AnalysisSettings {
            range_days: 7,
            ..Default::default()
        };
        repo.update_settings(&settings).unwrap();
        assert_eq!(repo.get_settings().unwrap().range_days, 7);

        repo.add_ignored_item("conflict", "abc", Some("accepted"))
            .unwrap();
        assert_eq!(
            repo.list_ignored_hashes("conflict").unwrap(),
            vec!["abc".to_string()]
        );

        repo.clear_cache().unwrap();
        assert!(repo.latest_raw_calendar(&source.id).unwrap().is_none());
        assert!(repo.list_events(&[]).unwrap().is_empty());
        assert!(repo.get_source(&source.id).unwrap().is_some());
    }
}
