import type { Language } from './i18n';
import { formatDurationHours, localeForLanguage } from './i18n';

export function formatTimeRange(start: string, end: string, language: Language = 'en') {
  const formatter = new Intl.DateTimeFormat(localeForLanguage(language), {
    hour: '2-digit',
    minute: '2-digit',
  });
  return `${formatter.format(new Date(start))}-${formatter.format(new Date(end))}`;
}

export function minutesToHours(minutes: number, language: Language = 'en') {
  return formatDurationHours(minutes, language);
}
