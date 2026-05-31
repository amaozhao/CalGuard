export type CalendarSourceDto = {
  id: string;
  name: string;
  kind: 'local_ics_file' | 'remote_ics_url' | string;
  color?: string | null;
  enabled: boolean;
  last_synced_at?: string | null;
  sync_status: 'idle' | 'syncing' | 'success' | 'failed' | string;
  error_message?: string | null;
};

export type ScoreDto = {
  score: number;
  grade: string;
  positive_reasons: string[];
  negative_reasons: string[];
};

export type ConflictDto = {
  id: string;
  starts_at: string;
  ends_at: string;
  severity: string;
  overlap_minutes: number;
  event_titles: string[];
  ignored: boolean;
};

export type FreeBlockDto = {
  id: string;
  starts_at: string;
  ends_at: string;
  duration_minutes: number;
  block_type: string;
};

export type OverloadedDayDto = {
  date: string;
  status: string;
  meeting_minutes: number;
  meeting_count: number;
  reasons: string[];
};

export type FocusMetricsDto = {
  deep_work_blocks: number;
  focus_minutes: number;
  longest_free_block_minutes: number;
  no_focus_days: string[];
  meeting_density_percent: number;
  fragmentation_score: number;
};

export type SuggestionDto = {
  id: string;
  text: string;
  reason: string;
  date?: string | null;
};

export type AnalysisReportDto = {
  schema_version: string;
  period_start: string;
  period_end: string;
  generated_at: string;
  score: ScoreDto;
  conflicts: ConflictDto[];
  free_blocks: FreeBlockDto[];
  overloaded_days: OverloadedDayDto[];
  focus_metrics: FocusMetricsDto;
  suggestions: SuggestionDto[];
};

export type SettingsDto = {
  range_days: number;
  timezone: string;
  workdays: number[];
  work_start: string;
  work_end: string;
  lunch_start: string;
  lunch_end: string;
  min_focus_minutes: number;
  overload_meeting_minutes: number;
  severe_overload_meeting_minutes: number;
  privacy_export_default: boolean;
};

export type ExportPrivacyDto = {
  include_event_titles: boolean;
  include_locations: boolean;
  include_descriptions: boolean;
  include_source_names: boolean;
};
