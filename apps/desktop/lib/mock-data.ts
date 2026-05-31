import type { AnalysisReportDto, CalendarSourceDto, SettingsDto } from './types';

export const defaultSettings: SettingsDto = {
  range_days: 14,
  timezone: 'UTC',
  workdays: [1, 2, 3, 4, 5],
  work_start: '09:00',
  work_end: '18:00',
  lunch_start: '12:00',
  lunch_end: '13:00',
  min_focus_minutes: 90,
  overload_meeting_minutes: 300,
  severe_overload_meeting_minutes: 420,
  privacy_export_default: false,
};

let browserSources: CalendarSourceDto[] = [];
let browserSettings = defaultSettings;

export function getBrowserSources() {
  return browserSources;
}

export function resetBrowserState() {
  browserSources = [];
  browserSettings = defaultSettings;
}

export function addBrowserSource(name: string, kind: CalendarSourceDto['kind']) {
  const source: CalendarSourceDto = {
    id: crypto.randomUUID(),
    name,
    kind,
    color: kind === 'remote_ics_url' ? '#0f766e' : '#2563eb',
    enabled: true,
    last_synced_at: new Date().toISOString(),
    sync_status: 'success',
    error_message: null,
  };
  browserSources = [...browserSources, source];
  return source;
}

export function updateBrowserSource(id: string, patch: Partial<CalendarSourceDto>) {
  browserSources = browserSources.map((source) => (source.id === id ? { ...source, ...patch } : source));
  return browserSources.find((source) => source.id === id);
}

export function removeBrowserSource(id: string) {
  browserSources = browserSources.filter((source) => source.id !== id);
}

export function getBrowserSettings() {
  return browserSettings;
}

export function setBrowserSettings(settings: SettingsDto) {
  browserSettings = settings;
  return browserSettings;
}

export function mockReport(rangeDays = 14): AnalysisReportDto {
  const now = new Date('2026-06-01T00:00:00Z');
  const end = new Date(now);
  end.setUTCDate(end.getUTCDate() + rangeDays);
  if (browserSources.length === 0) {
    return {
      schema_version: 'calguard.analysis.v1',
      period_start: now.toISOString(),
      period_end: end.toISOString(),
      generated_at: new Date().toISOString(),
      score: {
        score: 100,
        grade: 'Excellent',
        positive_reasons: ['No active calendar sources yet (+8)'],
        negative_reasons: [],
      },
      conflicts: [],
      free_blocks: [],
      overloaded_days: [],
      focus_metrics: {
        deep_work_blocks: 0,
        focus_minutes: 0,
        longest_free_block_minutes: 0,
        no_focus_days: [],
        meeting_density_percent: 0,
        fragmentation_score: 0,
      },
      suggestions: [],
    };
  }

  return {
    schema_version: 'calguard.analysis.v1',
    period_start: now.toISOString(),
    period_end: end.toISOString(),
    generated_at: new Date().toISOString(),
    score: {
      score: 72,
      grade: 'Warning',
      positive_reasons: ['At least one day has no meetings (+5)'],
      negative_reasons: ['Medium conflict at 2026-06-01 10:30 (-5)', '2026-06-02 has no deep work block (-8)'],
    },
    conflicts: [
      {
        id: 'mock-conflict-1',
        starts_at: '2026-06-01T10:30:00Z',
        ends_at: '2026-06-01T11:00:00Z',
        severity: 'Medium',
        overlap_minutes: 30,
        event_titles: ['Product Review', 'Candidate Interview'],
        ignored: false,
      },
    ],
    free_blocks: [
      {
        id: 'free-1',
        starts_at: '2026-06-01T13:30:00Z',
        ends_at: '2026-06-01T15:30:00Z',
        duration_minutes: 120,
        block_type: 'DeepWork',
      },
      {
        id: 'free-2',
        starts_at: '2026-06-02T09:00:00Z',
        ends_at: '2026-06-02T10:00:00Z',
        duration_minutes: 60,
        block_type: 'ShortGap',
      },
    ],
    overloaded_days: [
      {
        date: '2026-06-02',
        status: 'Risk',
        meeting_minutes: 360,
        meeting_count: 7,
        reasons: ['no deep work block', 'more than 5h of meetings'],
      },
    ],
    focus_metrics: {
      deep_work_blocks: 1,
      focus_minutes: 120,
      longest_free_block_minutes: 120,
      no_focus_days: ['2026-06-02'],
      meeting_density_percent: 42,
      fragmentation_score: 48,
    },
    suggestions: [
      {
        id: 'suggestion-1',
        text: 'Move Product Review away from Jun 1 10:30.',
        reason: 'It overlaps with Candidate Interview for 30 minutes.',
        date: '2026-06-01',
      },
      {
        id: 'suggestion-2',
        text: 'Protect Monday 13:30-15:30 as Focus Time.',
        reason: 'This is a 120 minute deep work block.',
        date: '2026-06-01',
      },
    ],
  };
}
