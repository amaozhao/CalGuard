'use client';

import { minutesToHours } from '../../lib/format-date';
import { localeForLanguage, translateTerm, useI18n } from '../../lib/i18n';
import type { AnalysisReportDto, SettingsDto } from '../../lib/types';

type HealthOverviewChartProps = {
  report: AnalysisReportDto;
  settings: SettingsDto;
};

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

function scoreTone(score: number) {
  if (score >= 75) return 'ok';
  if (score >= 60) return 'warn';
  return 'bad';
}

export function HealthOverviewChart({ report, settings }: HealthOverviewChartProps) {
  const { language, t } = useI18n();
  const activeConflicts = report.conflicts.filter((conflict) => !conflict.ignored).length;
  const score = report.score.score;
  const circumference = 2 * Math.PI * 48;
  const scoreOffset = circumference - (clamp(score, 0, 100) / 100) * circumference;
  const overloadByDate = new Map(report.overloaded_days.map((day) => [day.date, day]));
  const days = Array.from({ length: Math.min(settings.range_days, 7) }, (_, index) => {
    const date = new Date(report.period_start);
    date.setUTCDate(date.getUTCDate() + index);
    const key = date.toISOString().slice(0, 10);
    return { key, date, day: overloadByDate.get(key) };
  });
  const maxMeetingMinutes = Math.max(settings.severe_overload_meeting_minutes, ...days.map(({ day }) => day?.meeting_minutes ?? 0), 1);
  const focusTargetMinutes = Math.max(settings.min_focus_minutes * Math.max(settings.range_days, 1), 1);
  const focusRatio = clamp(report.focus_metrics.focus_minutes / focusTargetMinutes, 0, 1);
  const riskItems = [
    { label: t('conflicts'), value: activeConflicts, tone: activeConflicts > 0 ? 'bad' : 'ok' },
    { label: t('overloadedDays'), value: report.overloaded_days.length, tone: report.overloaded_days.length > 0 ? 'warn' : 'ok' },
    { label: t('noFocusDays'), value: report.focus_metrics.no_focus_days.length, tone: report.focus_metrics.no_focus_days.length > 0 ? 'warn' : 'ok' },
  ] as const;

  return (
    <section className="health-chart panel" aria-label={language === 'zh' ? '日历健康图表' : 'Calendar health chart'}>
      <div className="health-chart-main">
        <div className={`score-ring score-${scoreTone(score)}`}>
          <svg viewBox="0 0 120 120" aria-hidden="true">
            <circle className="score-ring-track" cx="60" cy="60" r="48" />
            <circle
              className="score-ring-value"
              cx="60"
              cy="60"
              r="48"
              strokeDasharray={circumference}
              strokeDashoffset={scoreOffset}
            />
          </svg>
          <div className="score-ring-label">
            <strong>{score}</strong>
            <span>/100</span>
          </div>
        </div>
        <div>
          <p className="eyebrow">{language === 'zh' ? '日历健康' : 'Calendar Health'}</p>
          <h2>{translateTerm(report.score.grade, language)}</h2>
          <p className="chart-summary">
            {activeConflicts} {t('activeConflicts')} · {minutesToHours(report.focus_metrics.focus_minutes, language)} {t('deepWork')}
          </p>
          <div className="focus-meter" aria-label={language === 'zh' ? '专注目标达成度' : 'Focus target progress'}>
            <span style={{ width: `${focusRatio * 100}%` }} />
          </div>
        </div>
      </div>

      <div className="load-chart">
        <div className="chart-heading">
          <h2>{language === 'zh' ? '7 天会议负载' : '7-Day Meeting Load'}</h2>
          <span>{language === 'zh' ? '过载阈值' : 'Overload'} {minutesToHours(settings.overload_meeting_minutes, language)}</span>
        </div>
        <div className="load-bars">
          {days.map(({ key, date, day }) => {
            const minutes = day?.meeting_minutes ?? 0;
            const height = 12 + (minutes / maxMeetingMinutes) * 88;
            const tone = minutes > settings.severe_overload_meeting_minutes ? 'bad' : minutes > settings.overload_meeting_minutes ? 'warn' : 'ok';
            return (
              <div className="load-day" key={key}>
                <div className="load-bar-shell" title={`${key}: ${minutesToHours(minutes, language)}`}>
                  <span className={`load-bar load-${tone}`} style={{ height: `${height}%` }} />
                </div>
                <strong>{date.toLocaleDateString(localeForLanguage(language), { weekday: 'short' })}</strong>
                <small>{minutes > 0 ? minutesToHours(minutes, language) : '-'}</small>
              </div>
            );
          })}
        </div>
      </div>

      <div className="risk-stack">
        <div className="chart-heading">
          <h2>{language === 'zh' ? '风险指纹' : 'Risk Signature'}</h2>
        </div>
        {riskItems.map((item) => (
          <div className={`risk-token risk-${item.tone}`} key={item.label}>
            <span>{item.label}</span>
            <strong>{item.value}</strong>
          </div>
        ))}
      </div>
    </section>
  );
}
