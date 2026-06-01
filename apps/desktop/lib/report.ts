import type { AnalysisReportDto, ConflictDto, ExportPrivacyDto } from './types';
import type { Language } from './i18n';
import { translateReportText } from './i18n';

export function filterConflicts(conflicts: ConflictDto[], severity: string) {
  if (severity === 'All') {
    return conflicts;
  }
  return conflicts.filter((conflict) => conflict.severity === severity);
}

export function privacySummary(privacy: ExportPrivacyDto, language: Language = 'en') {
  const hidden = [
    !privacy.include_event_titles ? (language === 'zh' ? '事件标题' : 'event titles') : null,
    !privacy.include_locations ? (language === 'zh' ? '地点' : 'locations') : null,
    !privacy.include_descriptions ? (language === 'zh' ? '描述' : 'descriptions') : null,
    !privacy.include_source_names ? (language === 'zh' ? '来源名称' : 'source names') : null,
  ].filter(Boolean);
  if (language === 'zh') {
    return hidden.length === 0 ? '完整报告' : `隐藏${hidden.join('、')}`;
  }
  return hidden.length === 0 ? 'Full report' : `Hides ${hidden.join(', ')}`;
}

export function markdownPreview(report: AnalysisReportDto, privacy: ExportPrivacyDto, language: Language = 'en') {
  const title = privacy.include_event_titles ? report.conflicts[0]?.event_titles[0] ?? 'No conflicts' : 'Busy Event';
  if (language === 'zh') {
    return [
      '# 日历健康报告',
      '',
      `评分：${report.score.score} / 100`,
      `等级：${translateReportText(report.score.grade, language)}`,
      `冲突：${report.conflicts.filter((conflict) => !conflict.ignored).length}`,
      `首要事项：${translateReportText(title, language)}`,
    ].join('\n');
  }
  return [
    '# Calendar Health Report',
    '',
    `Score: ${report.score.score} / 100`,
    `Grade: ${report.score.grade}`,
    `Conflicts: ${report.conflicts.filter((conflict) => !conflict.ignored).length}`,
    `Top item: ${title}`,
  ].join('\n');
}
