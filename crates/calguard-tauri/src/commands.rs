use crate::dto::*;
use crate::error::AppError;
use crate::state::AppState;
use calguard_core::{
    analyze_calendar as analyze_core, parse_ics, render_json_report, render_markdown_report,
    AnalysisInput, CalendarSource, CalendarSourceKind, ExportFormat, SyncStatus,
};
use calguard_store::repository::SourceUpdate;
use chrono::Utc;
use std::fs;
use std::time::Duration;
use tauri::State;

const MAX_ICS_BYTES: u64 = 20 * 1024 * 1024;
const REMOTE_TIMEOUT_SECONDS: u64 = 10;

pub fn command_handlers<R: tauri::Runtime>(
) -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        add_local_ics_source,
        add_remote_ics_source,
        list_calendar_sources,
        update_calendar_source,
        remove_calendar_source,
        sync_calendar_source,
        analyze_calendar,
        get_settings,
        update_settings,
        export_report,
        ignore_item,
        clear_cache
    ]
}

#[tauri::command]
pub async fn add_local_ics_source(
    input: AddLocalIcsSourceInput,
    state: State<'_, AppState>,
) -> Result<CalendarSourceDto, AppError> {
    let raw_ics =
        calguard_sync::read_local_ics(&input.path, MAX_ICS_BYTES).map_err(map_local_error)?;
    import_source(
        &state,
        input.name,
        CalendarSourceKind::LocalIcsFile { path: input.path },
        input.color,
        raw_ics,
    )
}

#[tauri::command]
pub async fn add_remote_ics_source(
    input: AddRemoteIcsSourceInput,
    state: State<'_, AppState>,
) -> Result<CalendarSourceDto, AppError> {
    let raw_ics = calguard_sync::fetch_remote_ics(
        &input.url,
        Duration::from_secs(REMOTE_TIMEOUT_SECONDS),
        MAX_ICS_BYTES as usize,
    )
    .await
    .map_err(map_remote_error)?;
    import_source(
        &state,
        input.name,
        CalendarSourceKind::RemoteIcsUrl { url: input.url },
        input.color,
        raw_ics,
    )
}

#[tauri::command]
pub async fn list_calendar_sources(
    state: State<'_, AppState>,
) -> Result<Vec<CalendarSourceDto>, AppError> {
    state.with_repository(|repo| {
        let sources = repo.list_sources().map_err(AppError::db)?;
        Ok(sources.into_iter().map(CalendarSourceDto::from).collect())
    })
}

#[tauri::command]
pub async fn update_calendar_source(
    input: UpdateCalendarSourceInput,
    state: State<'_, AppState>,
) -> Result<CalendarSourceDto, AppError> {
    state.with_repository(|repo| {
        let updated = repo
            .update_source(
                &input.source_id,
                SourceUpdate {
                    name: input.name.as_deref(),
                    enabled: input.enabled,
                    color: input.color.as_deref(),
                    ..Default::default()
                },
            )
            .map_err(AppError::db)?
            .ok_or_else(|| {
                AppError::new(
                    "SOURCE_NOT_FOUND",
                    "找不到该日历源。",
                    Some(input.source_id),
                )
            })?;
        Ok(CalendarSourceDto::from(updated))
    })
}

#[tauri::command]
pub async fn remove_calendar_source(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.with_repository(|repo| repo.remove_source(&source_id).map_err(AppError::db))
}

#[tauri::command]
pub async fn sync_calendar_source(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<SyncResultDto, AppError> {
    let source = state.with_repository(|repo| {
        repo.get_source(&source_id)
            .map_err(AppError::db)?
            .ok_or_else(|| {
                AppError::new(
                    "SOURCE_NOT_FOUND",
                    "找不到该日历源。",
                    Some(source_id.clone()),
                )
            })
    })?;

    let raw_ics_result = match source.kind {
        CalendarSourceKind::LocalIcsFile { path } => {
            calguard_sync::read_local_ics(path, MAX_ICS_BYTES).map_err(map_local_error)
        }
        CalendarSourceKind::RemoteIcsUrl { url } => calguard_sync::fetch_remote_ics(
            &url,
            Duration::from_secs(REMOTE_TIMEOUT_SECONDS),
            MAX_ICS_BYTES as usize,
        )
        .await
        .map_err(map_remote_error),
    };
    let raw_ics = match raw_ics_result {
        Ok(raw_ics) => raw_ics,
        Err(error) => {
            let details = error.to_string();
            let _ = state.with_repository(|repo| {
                repo.update_source(
                    &source.id,
                    SourceUpdate {
                        status: Some(SyncStatus::Failed),
                        error_message: Some(Some(details.as_str())),
                        ..Default::default()
                    },
                )
                .map_err(AppError::db)?;
                Ok(())
            });
            return Err(error);
        }
    };

    state.with_repository(|repo| {
        let raw = repo
            .store_raw_calendar(&source.id, &raw_ics)
            .map_err(AppError::db)?;
        let parsed = parse_ics(&raw_ics, &source.id);
        repo.replace_source_events(&source.id, &raw.id, &parsed.events)
            .map_err(AppError::db)?;
        repo.update_source(
            &source.id,
            SourceUpdate {
                status: Some(SyncStatus::Success),
                error_message: Some(None),
                last_synced_at: Some(Some(Utc::now())),
                ..Default::default()
            },
        )
        .map_err(AppError::db)?;
        Ok(SyncResultDto {
            source_id: source.id,
            event_count: parsed.events.len(),
            parse_issues: parsed
                .issues
                .into_iter()
                .map(|issue| format!("line {:?}: {}", issue.line, issue.message))
                .collect(),
        })
    })
}

#[tauri::command]
pub async fn analyze_calendar(
    input: AnalyzeCalendarInput,
    state: State<'_, AppState>,
) -> Result<AnalysisReportDto, AppError> {
    state.with_repository(|repo| {
        let mut settings = repo.get_settings().map_err(AppError::db)?;
        settings.range_days = input.range_days;
        settings.timezone = input.timezone;
        let events = repo.list_events(&input.source_ids).map_err(AppError::db)?;
        let ignored = repo.list_ignored_hashes("conflict").map_err(AppError::db)?;
        let enabled_source_ids = repo
            .list_sources()
            .map_err(AppError::db)?
            .into_iter()
            .filter(|source| source.enabled)
            .map(|source| source.id)
            .collect::<Vec<_>>();
        let report = analyze_core(AnalysisInput {
            events,
            settings,
            enabled_source_ids,
            ignored_conflict_hashes: ignored,
            generated_at: Utc::now(),
        });
        Ok(AnalysisReportDto::from(report))
    })
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<SettingsDto, AppError> {
    state.with_repository(|repo| {
        Ok(SettingsDto::from(
            repo.get_settings().map_err(AppError::db)?,
        ))
    })
}

#[tauri::command]
pub async fn update_settings(
    input: UpdateSettingsInput,
    state: State<'_, AppState>,
) -> Result<SettingsDto, AppError> {
    let settings = calguard_core::AnalysisSettings::try_from(input.settings.clone())?;
    state.with_repository(|repo| {
        repo.update_settings(&settings).map_err(AppError::db)?;
        Ok(input.settings)
    })
}

#[tauri::command]
pub async fn export_report(
    input: ExportReportInput,
    state: State<'_, AppState>,
) -> Result<ExportReportResultDto, AppError> {
    state.with_repository(|repo| {
        let mut settings = repo.get_settings().map_err(AppError::db)?;
        settings.range_days = input.range_days;
        let events = repo.list_events(&[]).map_err(AppError::db)?;
        let ignored = repo.list_ignored_hashes("conflict").map_err(AppError::db)?;
        let enabled_source_ids = repo
            .list_sources()
            .map_err(AppError::db)?
            .into_iter()
            .filter(|source| source.enabled)
            .map(|source| source.id)
            .collect::<Vec<_>>();
        let report = analyze_core(AnalysisInput {
            events,
            settings,
            enabled_source_ids,
            ignored_conflict_hashes: ignored,
            generated_at: Utc::now(),
        });
        let format = export_format(&input.format)?;
        let privacy_mode = !input.privacy.include_event_titles || !input.privacy.include_locations;
        let body = match format {
            ExportFormat::Markdown => {
                render_markdown_report(&report, &input.privacy.clone().into())
            }
            ExportFormat::Json => render_json_report(&report, &input.privacy.clone().into())
                .map_err(|err| {
                    AppError::new("EXPORT_FAILED", "导出报告失败。", Some(err.to_string()))
                })?,
        };
        fs::write(&input.file_path, body.as_bytes()).map_err(|err| {
            AppError::new("EXPORT_FAILED", "导出报告失败。", Some(err.to_string()))
        })?;
        repo.record_export(
            &input.format,
            &input.file_path,
            report.period_start,
            report.period_end,
            privacy_mode,
        )
        .map_err(AppError::db)?;
        Ok(ExportReportResultDto {
            file_path: input.file_path,
            bytes_written: body.len(),
        })
    })
}

#[tauri::command]
pub async fn ignore_item(
    input: IgnoreItemInput,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.with_repository(|repo| {
        repo.add_ignored_item(&input.kind, &input.target_hash, input.reason.as_deref())
            .map_err(AppError::db)
    })
}

#[tauri::command]
pub async fn clear_cache(state: State<'_, AppState>) -> Result<(), AppError> {
    state.with_repository(|repo| repo.clear_cache().map_err(AppError::db))
}

fn import_source(
    state: &State<'_, AppState>,
    name: String,
    kind: CalendarSourceKind,
    color: Option<String>,
    raw_ics: String,
) -> Result<CalendarSourceDto, AppError> {
    state.with_repository(|repo| {
        let now = Utc::now();
        let source = repo
            .add_source(CalendarSource {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                kind,
                color,
                enabled: true,
                created_at: now,
                updated_at: now,
                last_synced_at: Some(now),
                sync_status: SyncStatus::Success,
                error_message: None,
            })
            .map_err(AppError::db)?;
        let raw = repo
            .store_raw_calendar(&source.id, &raw_ics)
            .map_err(AppError::db)?;
        let parsed = parse_ics(&raw_ics, &source.id);
        if parsed.events.is_empty() && !parsed.issues.is_empty() {
            let details = parsed
                .issues
                .iter()
                .map(|issue| format!("line {:?}: {}", issue.line, issue.message))
                .collect::<Vec<_>>()
                .join("; ");
            repo.update_source(
                &source.id,
                SourceUpdate {
                    status: Some(SyncStatus::Failed),
                    error_message: Some(Some(details.as_str())),
                    ..Default::default()
                },
            )
            .map_err(AppError::db)?;
            return Err(AppError::parse_failed(details));
        }
        repo.replace_source_events(&source.id, &raw.id, &parsed.events)
            .map_err(AppError::db)?;
        Ok(CalendarSourceDto::from(source))
    })
}

fn map_local_error(error: calguard_sync::local_ics::LocalIcsError) -> AppError {
    match error {
        calguard_sync::local_ics::LocalIcsError::FileNotFound => {
            AppError::file_not_found(error.to_string())
        }
        calguard_sync::local_ics::LocalIcsError::FileTooLarge => {
            AppError::file_too_large(error.to_string())
        }
        calguard_sync::local_ics::LocalIcsError::InvalidExtension => AppError::new(
            "INVALID_FILE_TYPE",
            "请选择 .ics 日历文件。",
            Some(error.to_string()),
        ),
        calguard_sync::local_ics::LocalIcsError::Io(_) => AppError::new(
            "FILE_READ_FAILED",
            "读取日历文件失败。",
            Some(error.to_string()),
        ),
    }
}

fn map_remote_error(error: calguard_sync::remote_ics::RemoteIcsError) -> AppError {
    match error {
        calguard_sync::remote_ics::RemoteIcsError::InvalidUrl => {
            AppError::invalid_url(error.to_string())
        }
        calguard_sync::remote_ics::RemoteIcsError::Timeout => AppError::remote_timeout(),
        calguard_sync::remote_ics::RemoteIcsError::TooLarge => {
            AppError::file_too_large(error.to_string())
        }
        calguard_sync::remote_ics::RemoteIcsError::FetchFailed(_) => {
            AppError::remote_failed(error.to_string())
        }
    }
}
