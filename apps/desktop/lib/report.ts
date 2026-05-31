import type { AnalysisReportDto, ConflictDto, ExportPrivacyDto } from './types';

export function filterConflicts(conflicts: ConflictDto[], severity: string) {
  if (severity === 'All') {
    return conflicts;
  }
  return conflicts.filter((conflict) => conflict.severity === severity);
}

export function privacySummary(privacy: ExportPrivacyDto) {
  const hidden = [
    !privacy.include_event_titles ? 'event titles' : null,
    !privacy.include_locations ? 'locations' : null,
    !privacy.include_descriptions ? 'descriptions' : null,
    !privacy.include_source_names ? 'source names' : null,
  ].filter(Boolean);
  return hidden.length === 0 ? 'Full report' : `Hides ${hidden.join(', ')}`;
}

export function markdownPreview(report: AnalysisReportDto, privacy: ExportPrivacyDto) {
  const title = privacy.include_event_titles ? report.conflicts[0]?.event_titles[0] ?? 'No conflicts' : 'Busy Event';
  return [
    '# Calendar Health Report',
    '',
    `Score: ${report.score.score} / 100`,
    `Grade: ${report.score.grade}`,
    `Conflicts: ${report.conflicts.filter((conflict) => !conflict.ignored).length}`,
    `Top item: ${title}`,
  ].join('\n');
}
