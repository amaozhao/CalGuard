'use client';

import { addLocalIcsSource, addRemoteIcsSource } from './tauri';

export type SourceMode = 'local' | 'remote';

export type CalendarSourceDraft = {
  name: string;
  mode: SourceMode;
  path: string;
  url: string;
};

export const initialSourceDraft: CalendarSourceDraft = {
  name: 'My Calendar',
  mode: 'local',
  path: '',
  url: '',
};

export async function addCalendarSourceFromDraft(
  draft: CalendarSourceDraft,
  options: { requireLocalIcsExtension?: boolean } = {},
) {
  if (draft.mode === 'local') {
    if (!draft.path || (options.requireLocalIcsExtension && !draft.path.toLowerCase().endsWith('.ics'))) {
      throw new Error('Select an .ics file');
    }
    return addLocalIcsSource({ name: draft.name, path: draft.path, color: '#2563eb' });
  }

  if (!draft.url.startsWith('http://') && !draft.url.startsWith('https://')) {
    throw new Error('Use an HTTP or HTTPS ICS URL');
  }
  return addRemoteIcsSource({ name: draft.name, url: draft.url, color: '#0f766e' });
}
