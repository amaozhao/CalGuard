export function formatDateTime(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}

export function formatTimeRange(start: string, end: string) {
  const formatter = new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    minute: '2-digit',
  });
  return `${formatter.format(new Date(start))}-${formatter.format(new Date(end))}`;
}

export function minutesToHours(minutes: number) {
  return `${(minutes / 60).toFixed(1)}h`;
}
