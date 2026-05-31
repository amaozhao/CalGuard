use rusqlite::Connection;

pub fn migrate(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS calendar_sources (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            config_json TEXT NOT NULL,
            color TEXT,
            enabled INTEGER NOT NULL DEFAULT 1,
            sync_status TEXT NOT NULL DEFAULT 'idle',
            last_error TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_synced_at TEXT
        );

        CREATE TABLE IF NOT EXISTS raw_calendars (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL,
            raw_ics TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            etag TEXT,
            last_modified TEXT,
            imported_at TEXT NOT NULL,
            FOREIGN KEY(source_id) REFERENCES calendar_sources(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL,
            uid TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            location TEXT,
            starts_at TEXT NOT NULL,
            ends_at TEXT NOT NULL,
            timezone TEXT,
            is_all_day INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL,
            transparency TEXT NOT NULL,
            recurrence_rule TEXT,
            excluded_dates_json TEXT,
            recurrence_id TEXT,
            raw_calendar_id TEXT,
            FOREIGN KEY(source_id) REFERENCES calendar_sources(id) ON DELETE CASCADE,
            FOREIGN KEY(raw_calendar_id) REFERENCES raw_calendars(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS ignored_items (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            target_hash TEXT NOT NULL,
            reason TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value_json TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS report_exports (
            id TEXT PRIMARY KEY,
            format TEXT NOT NULL,
            file_path TEXT NOT NULL,
            period_start TEXT NOT NULL,
            period_end TEXT NOT NULL,
            privacy_mode INTEGER NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )
}
