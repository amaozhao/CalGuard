use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

pub type CalendarSourceId = String;
pub type EventId = String;
pub type EventInstanceId = String;
pub type ConflictId = String;
pub type FreeBlockId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CalendarSourceKind {
    LocalIcsFile { path: String },
    RemoteIcsUrl { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Idle,
    Syncing,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarSource {
    pub id: CalendarSourceId,
    pub name: String,
    pub kind: CalendarSourceKind,
    pub color: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawCalendar {
    pub id: String,
    pub source_id: CalendarSourceId,
    pub raw_ics: String,
    pub content_hash: String,
    pub imported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventStatus {
    #[default]
    Confirmed,
    Tentative,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Transparency {
    #[default]
    Opaque,
    Transparent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub id: EventId,
    pub source_id: CalendarSourceId,
    pub uid: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub timezone: Option<String>,
    pub is_all_day: bool,
    pub status: EventStatus,
    pub transparency: Transparency,
    pub recurrence_rule: Option<String>,
    pub excluded_dates: Vec<DateTime<Utc>>,
    pub recurrence_id: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventInstance {
    pub id: EventInstanceId,
    pub event_id: EventId,
    pub source_id: CalendarSourceId,
    pub uid: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub is_all_day: bool,
    pub status: EventStatus,
    pub transparency: Transparency,
}

impl EventInstance {
    pub fn is_busy(&self, settings: &AnalysisSettings) -> bool {
        if self.status == EventStatus::Cancelled {
            return false;
        }
        if self.transparency == Transparency::Transparent && !settings.include_transparent_as_busy {
            return false;
        }
        if self.is_all_day && !settings.include_all_day_as_busy {
            return false;
        }
        self.starts_at < self.ends_at
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Conflict {
    pub id: ConflictId,
    pub event_ids: Vec<EventInstanceId>,
    pub events: Vec<EventInstance>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub overlap_minutes: i64,
    pub severity: ConflictSeverity,
    pub reason: String,
    pub ignored: bool,
    pub ignore_hash: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum FreeBlockType {
    DeepWork,
    ShortGap,
    MicroGap,
    Lunch,
    OutsideWork,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FreeBlock {
    pub id: FreeBlockId,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub duration_minutes: i64,
    pub block_type: FreeBlockType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum HealthGrade {
    Excellent,
    Good,
    Warning,
    Poor,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreReason {
    pub id: String,
    pub label: String,
    pub points: i16,
    pub date: Option<String>,
    pub event_ids: Vec<EventInstanceId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthScore {
    pub score: u8,
    pub grade: HealthGrade,
    pub positive_reasons: Vec<ScoreReason>,
    pub negative_reasons: Vec<ScoreReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FocusMetrics {
    pub deep_work_blocks: usize,
    pub focus_minutes: i64,
    pub longest_free_block_minutes: i64,
    pub no_focus_days: Vec<String>,
    pub meeting_density_percent: u8,
    pub fragmentation_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverloadedDay {
    pub date: String,
    pub status: OverloadStatus,
    pub meeting_minutes: i64,
    pub meeting_count: usize,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum OverloadStatus {
    Risk,
    Overloaded,
    SeverelyOverloaded,
    BoundaryRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Suggestion {
    pub id: String,
    pub kind: SuggestionKind,
    pub text: String,
    pub reason: String,
    pub date: Option<String>,
    pub event_ids: Vec<EventInstanceId>,
    pub free_block_id: Option<FreeBlockId>,
    pub ignored: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum SuggestionKind {
    MoveEvent,
    AddFocusBlock,
    AddBuffer,
    DeclineOrShortenMeeting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisReport {
    pub schema_version: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub generated_at: DateTime<Utc>,
    pub score: HealthScore,
    pub conflicts: Vec<Conflict>,
    pub free_blocks: Vec<FreeBlock>,
    pub overloaded_days: Vec<OverloadedDay>,
    pub focus_metrics: FocusMetrics,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisSettings {
    pub range_days: u32,
    pub timezone: String,
    pub workdays: Vec<u32>,
    pub work_start: NaiveTime,
    pub work_end: NaiveTime,
    pub lunch_start: NaiveTime,
    pub lunch_end: NaiveTime,
    pub min_focus_minutes: i64,
    pub min_short_gap_minutes: i64,
    pub overload_meeting_minutes: i64,
    pub severe_overload_meeting_minutes: i64,
    pub overload_meeting_count: usize,
    pub include_all_day_as_busy: bool,
    pub include_transparent_as_busy: bool,
    pub max_recurring_instances: usize,
}

impl Default for AnalysisSettings {
    fn default() -> Self {
        Self {
            range_days: 14,
            timezone: "UTC".to_string(),
            workdays: vec![1, 2, 3, 4, 5],
            work_start: NaiveTime::from_hms_opt(9, 0, 0).expect("valid default time"),
            work_end: NaiveTime::from_hms_opt(18, 0, 0).expect("valid default time"),
            lunch_start: NaiveTime::from_hms_opt(12, 0, 0).expect("valid default time"),
            lunch_end: NaiveTime::from_hms_opt(13, 0, 0).expect("valid default time"),
            min_focus_minutes: 90,
            min_short_gap_minutes: 30,
            overload_meeting_minutes: 300,
            severe_overload_meeting_minutes: 420,
            overload_meeting_count: 8,
            include_all_day_as_busy: false,
            include_transparent_as_busy: false,
            max_recurring_instances: 1000,
        }
    }
}

impl AnalysisSettings {
    pub fn normalized_range_days(&self) -> u32 {
        self.range_days.clamp(1, 30)
    }

    pub fn is_workday_number(&self, weekday_number: u32) -> bool {
        self.workdays.contains(&weekday_number)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisInput {
    pub events: Vec<Event>,
    pub settings: AnalysisSettings,
    pub enabled_source_ids: Vec<CalendarSourceId>,
    pub ignored_conflict_hashes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportPrivacyOptions {
    pub include_event_titles: bool,
    pub include_locations: bool,
    pub include_descriptions: bool,
    pub include_source_names: bool,
}

impl Default for ReportPrivacyOptions {
    fn default() -> Self {
        Self {
            include_event_titles: true,
            include_locations: false,
            include_descriptions: false,
            include_source_names: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Markdown,
    Json,
}
