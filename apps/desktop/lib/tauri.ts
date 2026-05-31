'use client';

import {
  addBrowserSource,
  getBrowserSettings,
  getBrowserSources,
  mockReport,
  removeBrowserSource,
  setBrowserSettings,
  updateBrowserSource,
} from './mock-data';
import type { AnalysisReportDto, CalendarSourceDto, ExportPrivacyDto, SettingsDto } from './types';

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

export async function chooseIcsFile() {
  if (!isTauriRuntime()) {
    return '/fixtures/simple.ics';
  }
  const { open } = await import('@tauri-apps/plugin-dialog');
  const selected = await open({
    title: 'Select ICS file',
    filters: [{ name: 'iCalendar', extensions: ['ics'] }],
    multiple: false,
  });
  return typeof selected === 'string' ? selected : null;
}

export async function chooseReportPath(format: 'markdown' | 'json') {
  if (!isTauriRuntime()) {
    return `/tmp/calguard-report.${format === 'markdown' ? 'md' : 'json'}`;
  }
  const { save } = await import('@tauri-apps/plugin-dialog');
  return save({
    title: 'Save report',
    filters: [{ name: format === 'markdown' ? 'Markdown' : 'JSON', extensions: [format === 'markdown' ? 'md' : 'json'] }],
  });
}

export async function listCalendarSources(): Promise<CalendarSourceDto[]> {
  if (!isTauriRuntime()) {
    return getBrowserSources();
  }
  return invokeCommand('list_calendar_sources');
}

export async function addLocalIcsSource(input: { name: string; path: string; color?: string | null }) {
  if (!isTauriRuntime()) {
    return addBrowserSource(input.name, 'local_ics_file');
  }
  return invokeCommand<CalendarSourceDto>('add_local_ics_source', { input });
}

export async function addRemoteIcsSource(input: { name: string; url: string; color?: string | null }) {
  if (!isTauriRuntime()) {
    return addBrowserSource(input.name, 'remote_ics_url');
  }
  return invokeCommand<CalendarSourceDto>('add_remote_ics_source', { input });
}

export async function updateCalendarSource(input: {
  source_id: string;
  name?: string;
  enabled?: boolean;
  color?: string;
}) {
  if (!isTauriRuntime()) {
    const source = updateBrowserSource(input.source_id, {
      name: input.name,
      enabled: input.enabled,
      color: input.color,
    });
    if (!source) {
      throw new Error('Source not found');
    }
    return source;
  }
  return invokeCommand<CalendarSourceDto>('update_calendar_source', { input });
}

export async function removeCalendarSource(sourceId: string) {
  if (!isTauriRuntime()) {
    removeBrowserSource(sourceId);
    return;
  }
  return invokeCommand<void>('remove_calendar_source', { sourceId });
}

export async function syncCalendarSource(sourceId: string) {
  if (!isTauriRuntime()) {
    return { source_id: sourceId, event_count: 2, parse_issues: [] };
  }
  return invokeCommand('sync_calendar_source', { sourceId });
}

export async function analyzeCalendar(rangeDays: number, sourceIds: string[] = [], timezone = 'UTC'): Promise<AnalysisReportDto> {
  if (!isTauriRuntime()) {
    return mockReport(rangeDays);
  }
  return invokeCommand('analyze_calendar', {
    input: { range_days: rangeDays, source_ids: sourceIds, timezone },
  });
}

export async function getSettings(): Promise<SettingsDto> {
  if (!isTauriRuntime()) {
    return getBrowserSettings();
  }
  return invokeCommand('get_settings');
}

export async function updateSettings(settings: SettingsDto) {
  if (!isTauriRuntime()) {
    return setBrowserSettings(settings);
  }
  return invokeCommand<SettingsDto>('update_settings', { input: { settings } });
}

export async function exportReport(input: {
  file_path: string;
  format: 'markdown' | 'json';
  range_days: number;
  privacy: ExportPrivacyDto;
}) {
  if (!isTauriRuntime()) {
    return { file_path: input.file_path, bytes_written: 512 };
  }
  return invokeCommand<{ file_path: string; bytes_written: number }>('export_report', { input });
}

export async function ignoreConflict(targetHash: string) {
  if (!isTauriRuntime()) {
    return;
  }
  return invokeCommand<void>('ignore_item', {
    input: { kind: 'conflict', target_hash: targetHash, reason: 'Ignored in UI' },
  });
}

export async function clearCache() {
  if (!isTauriRuntime()) {
    return;
  }
  return invokeCommand<void>('clear_cache');
}
