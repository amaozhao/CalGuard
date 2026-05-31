use calguard_core::{
    AnalysisReport, AnalysisSettings, CalendarSource, Conflict, EventInstance, ExportFormat,
    FocusMetrics, FreeBlock, OverloadedDay, ReportPrivacyOptions, ScoreReason, Suggestion,
};
use chrono::{Duration, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSourceDto {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub color: Option<String>,
    pub enabled: bool,
    pub last_synced_at: Option<String>,
    pub sync_status: String,
    pub error_message: Option<String>,
}

impl From<CalendarSource> for CalendarSourceDto {
    fn from(source: CalendarSource) -> Self {
        let kind = match source.kind {
            calguard_core::CalendarSourceKind::LocalIcsFile { .. } => "local_ics_file",
            calguard_core::CalendarSourceKind::RemoteIcsUrl { .. } => "remote_ics_url",
        };
        Self {
            id: source.id,
            name: source.name,
            kind: kind.to_string(),
            color: source.color,
            enabled: source.enabled,
            last_synced_at: source.last_synced_at.map(|date| date.to_rfc3339()),
            sync_status: format!("{:?}", source.sync_status).to_ascii_lowercase(),
            error_message: source.error_message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddLocalIcsSourceInput {
    pub name: String,
    pub path: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRemoteIcsSourceInput {
    pub name: String,
    pub url: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCalendarSourceInput {
    pub source_id: String,
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResultDto {
    pub source_id: String,
    pub event_count: usize,
    pub parse_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeCalendarInput {
    pub range_days: u32,
    pub source_ids: Vec<String>,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReportDto {
    pub schema_version: String,
    pub period_start: String,
    pub period_end: String,
    pub generated_at: String,
    pub score: ScoreDto,
    pub conflicts: Vec<ConflictDto>,
    pub free_blocks: Vec<FreeBlockDto>,
    pub overloaded_days: Vec<OverloadedDayDto>,
    pub focus_metrics: FocusMetricsDto,
    pub suggestions: Vec<SuggestionDto>,
}

impl AnalysisReportDto {
    pub fn empty(range_days: u32) -> Self {
        let now = Utc::now();
        Self {
            schema_version: "calguard.analysis.v1".to_string(),
            period_start: now.to_rfc3339(),
            period_end: (now + Duration::days(range_days as i64)).to_rfc3339(),
            generated_at: now.to_rfc3339(),
            score: ScoreDto {
                score: 100,
                grade: "Excellent".to_string(),
                positive_reasons: vec!["No active calendar sources yet".to_string()],
                negative_reasons: Vec::new(),
            },
            conflicts: Vec::new(),
            free_blocks: Vec::new(),
            overloaded_days: Vec::new(),
            focus_metrics: FocusMetricsDto::default(),
            suggestions: Vec::new(),
        }
    }
}

impl From<AnalysisReport> for AnalysisReportDto {
    fn from(report: AnalysisReport) -> Self {
        Self {
            schema_version: report.schema_version,
            period_start: report.period_start.to_rfc3339(),
            period_end: report.period_end.to_rfc3339(),
            generated_at: report.generated_at.to_rfc3339(),
            score: ScoreDto {
                score: report.score.score,
                grade: format!("{:?}", report.score.grade),
                positive_reasons: report
                    .score
                    .positive_reasons
                    .iter()
                    .map(reason_to_text)
                    .collect(),
                negative_reasons: report
                    .score
                    .negative_reasons
                    .iter()
                    .map(reason_to_text)
                    .collect(),
            },
            conflicts: report
                .conflicts
                .into_iter()
                .map(ConflictDto::from)
                .collect(),
            free_blocks: report
                .free_blocks
                .into_iter()
                .map(FreeBlockDto::from)
                .collect(),
            overloaded_days: report
                .overloaded_days
                .into_iter()
                .map(OverloadedDayDto::from)
                .collect(),
            focus_metrics: FocusMetricsDto::from(report.focus_metrics),
            suggestions: report
                .suggestions
                .into_iter()
                .map(SuggestionDto::from)
                .collect(),
        }
    }
}

fn reason_to_text(reason: &ScoreReason) -> String {
    format!("{} ({:+})", reason.label, reason.points)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDto {
    pub score: u8,
    pub grade: String,
    pub positive_reasons: Vec<String>,
    pub negative_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDto {
    pub id: String,
    pub starts_at: String,
    pub ends_at: String,
    pub severity: String,
    pub overlap_minutes: i64,
    pub event_titles: Vec<String>,
    pub ignored: bool,
}

impl From<Conflict> for ConflictDto {
    fn from(conflict: Conflict) -> Self {
        Self {
            id: conflict.id,
            starts_at: conflict.starts_at.to_rfc3339(),
            ends_at: conflict.ends_at.to_rfc3339(),
            severity: format!("{:?}", conflict.severity),
            overlap_minutes: conflict.overlap_minutes,
            event_titles: conflict
                .events
                .into_iter()
                .map(|event| event_title(&event))
                .collect(),
            ignored: conflict.ignored,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeBlockDto {
    pub id: String,
    pub starts_at: String,
    pub ends_at: String,
    pub duration_minutes: i64,
    pub block_type: String,
}

impl From<FreeBlock> for FreeBlockDto {
    fn from(block: FreeBlock) -> Self {
        Self {
            id: block.id,
            starts_at: block.starts_at.to_rfc3339(),
            ends_at: block.ends_at.to_rfc3339(),
            duration_minutes: block.duration_minutes,
            block_type: format!("{:?}", block.block_type),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverloadedDayDto {
    pub date: String,
    pub status: String,
    pub meeting_minutes: i64,
    pub meeting_count: usize,
    pub reasons: Vec<String>,
}

impl From<OverloadedDay> for OverloadedDayDto {
    fn from(day: OverloadedDay) -> Self {
        Self {
            date: day.date,
            status: format!("{:?}", day.status),
            meeting_minutes: day.meeting_minutes,
            meeting_count: day.meeting_count,
            reasons: day.reasons,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FocusMetricsDto {
    pub deep_work_blocks: usize,
    pub focus_minutes: i64,
    pub longest_free_block_minutes: i64,
    pub no_focus_days: Vec<String>,
    pub meeting_density_percent: u8,
    pub fragmentation_score: u8,
}

impl From<FocusMetrics> for FocusMetricsDto {
    fn from(metrics: FocusMetrics) -> Self {
        Self {
            deep_work_blocks: metrics.deep_work_blocks,
            focus_minutes: metrics.focus_minutes,
            longest_free_block_minutes: metrics.longest_free_block_minutes,
            no_focus_days: metrics.no_focus_days,
            meeting_density_percent: metrics.meeting_density_percent,
            fragmentation_score: metrics.fragmentation_score,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionDto {
    pub id: String,
    pub text: String,
    pub reason: String,
    pub date: Option<String>,
}

impl From<Suggestion> for SuggestionDto {
    fn from(suggestion: Suggestion) -> Self {
        Self {
            id: suggestion.id,
            text: suggestion.text,
            reason: suggestion.reason,
            date: suggestion.date,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsDto {
    pub range_days: u32,
    pub timezone: String,
    pub workdays: Vec<u32>,
    pub work_start: String,
    pub work_end: String,
    pub lunch_start: String,
    pub lunch_end: String,
    pub min_focus_minutes: i64,
    pub overload_meeting_minutes: i64,
    pub severe_overload_meeting_minutes: i64,
    pub privacy_export_default: bool,
}

impl Default for SettingsDto {
    fn default() -> Self {
        Self {
            range_days: 14,
            timezone: "UTC".to_string(),
            workdays: vec![1, 2, 3, 4, 5],
            work_start: "09:00".to_string(),
            work_end: "18:00".to_string(),
            lunch_start: "12:00".to_string(),
            lunch_end: "13:00".to_string(),
            min_focus_minutes: 90,
            overload_meeting_minutes: 300,
            severe_overload_meeting_minutes: 420,
            privacy_export_default: false,
        }
    }
}

impl TryFrom<SettingsDto> for AnalysisSettings {
    type Error = crate::error::AppError;

    fn try_from(settings: SettingsDto) -> Result<Self, Self::Error> {
        Ok(Self {
            range_days: settings.range_days,
            timezone: settings.timezone,
            workdays: settings.workdays,
            work_start: parse_time(&settings.work_start)?,
            work_end: parse_time(&settings.work_end)?,
            lunch_start: parse_time(&settings.lunch_start)?,
            lunch_end: parse_time(&settings.lunch_end)?,
            min_focus_minutes: settings.min_focus_minutes,
            min_short_gap_minutes: 30,
            overload_meeting_minutes: settings.overload_meeting_minutes,
            severe_overload_meeting_minutes: settings.severe_overload_meeting_minutes,
            overload_meeting_count: 8,
            include_all_day_as_busy: false,
            include_transparent_as_busy: false,
            max_recurring_instances: 1000,
        })
    }
}

impl From<AnalysisSettings> for SettingsDto {
    fn from(settings: AnalysisSettings) -> Self {
        Self {
            range_days: settings.range_days,
            timezone: settings.timezone,
            workdays: settings.workdays,
            work_start: settings.work_start.format("%H:%M").to_string(),
            work_end: settings.work_end.format("%H:%M").to_string(),
            lunch_start: settings.lunch_start.format("%H:%M").to_string(),
            lunch_end: settings.lunch_end.format("%H:%M").to_string(),
            min_focus_minutes: settings.min_focus_minutes,
            overload_meeting_minutes: settings.overload_meeting_minutes,
            severe_overload_meeting_minutes: settings.severe_overload_meeting_minutes,
            privacy_export_default: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettingsInput {
    pub settings: SettingsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportReportInput {
    pub file_path: String,
    pub format: String,
    pub range_days: u32,
    pub privacy: ExportPrivacyDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPrivacyDto {
    pub include_event_titles: bool,
    pub include_locations: bool,
    pub include_descriptions: bool,
    pub include_source_names: bool,
}

impl From<ExportPrivacyDto> for ReportPrivacyOptions {
    fn from(privacy: ExportPrivacyDto) -> Self {
        Self {
            include_event_titles: privacy.include_event_titles,
            include_locations: privacy.include_locations,
            include_descriptions: privacy.include_descriptions,
            include_source_names: privacy.include_source_names,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportReportResultDto {
    pub file_path: String,
    pub bytes_written: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreItemInput {
    pub kind: String,
    pub target_hash: String,
    pub reason: Option<String>,
}

pub fn export_format(value: &str) -> Result<ExportFormat, crate::error::AppError> {
    match value {
        "markdown" | "md" => Ok(ExportFormat::Markdown),
        "json" => Ok(ExportFormat::Json),
        other => Err(crate::error::AppError::new(
            "INVALID_EXPORT_FORMAT",
            "导出格式无效。",
            Some(other.to_string()),
        )),
    }
}

fn parse_time(value: &str) -> Result<NaiveTime, crate::error::AppError> {
    NaiveTime::parse_from_str(value, "%H:%M").map_err(|err| {
        crate::error::AppError::new(
            "INVALID_SETTINGS",
            "设置中的时间格式无效。",
            Some(err.to_string()),
        )
    })
}

fn event_title(event: &EventInstance) -> String {
    event.title.clone()
}
