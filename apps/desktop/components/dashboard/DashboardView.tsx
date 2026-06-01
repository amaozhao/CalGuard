'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { EmptyState } from '../common/EmptyState';
import { MetricCard } from '../common/MetricCard';
import { StatusPill } from '../common/StatusPill';
import { HealthOverviewChart } from './HealthOverviewChart';
import { formatTimeRange, minutesToHours } from '../../lib/format-date';
import { formatDayCount, localeForLanguage, translateReportText, translateTerm, useI18n } from '../../lib/i18n';
import { analyzeCalendar, getSettings, listCalendarSources, syncCalendarSource } from '../../lib/tauri';
import type { AnalysisReportDto, CalendarSourceDto, SettingsDto } from '../../lib/types';

export function DashboardView() {
  const { language, t } = useI18n();
  const [sources, setSources] = useState<CalendarSourceDto[]>([]);
  const [settings, setSettings] = useState<SettingsDto | null>(null);
  const [report, setReport] = useState<AnalysisReportDto | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  async function load(rangeDays?: number) {
    setLoading(true);
    setError(null);
    try {
      const [nextSources, nextSettings] = await Promise.all([listCalendarSources(), getSettings()]);
      const nextReport = await analyzeCalendar(rangeDays ?? nextSettings.range_days, [], nextSettings.timezone);
      setSources(nextSources);
      setSettings({ ...nextSettings, range_days: rangeDays ?? nextSettings.range_days });
      setReport(nextReport);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load dashboard');
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void load();
  }, []);

  async function refreshRemote() {
    const remoteSources = sources.filter((source) => source.enabled && source.kind === 'remote_ics_url');
    await Promise.all(remoteSources.map((source) => syncCalendarSource(source.id)));
    await load(settings?.range_days);
  }

  if (loading && !report) {
    return <section className="panel">{t('loadingDashboard')}</section>;
  }

  if (sources.length === 0) {
    return <EmptyState />;
  }

  if (!report || !settings) {
    return <section className="panel">{t('noAnalysisReport')}</section>;
  }

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <h1>{t('dashboard')}</h1>
          <p>
            {new Date(report.period_start).toLocaleDateString(localeForLanguage(language))} {' '}
            {language === 'zh' ? '至' : 'to'} {new Date(report.period_end).toLocaleDateString(localeForLanguage(language))}
          </p>
        </div>
        <div className="toolbar">
          {[7, 14, 30].map((days) => (
            <button
              key={days}
              className={settings.range_days === days ? 'segmented active' : 'segmented'}
              type="button"
              onClick={() => void load(days)}
            >
              {formatDayCount(days, language)}
            </button>
          ))}
          <button className="button" type="button" onClick={() => void refreshRemote()}>
            {t('refresh')}
          </button>
          <Link className="button primary" href="/reports/">
            {t('export')}
          </Link>
        </div>
      </header>

      {error ? <div className="error-banner">{translateReportText(error, language)}</div> : null}

      <HealthOverviewChart report={report} settings={settings} />

      <section className="metrics-grid">
        <Link href="/dashboard/">
          <MetricCard label={t('healthScore')} value={`${report.score.score}`} detail={translateTerm(report.score.grade, language)} />
        </Link>
        <Link href="/conflicts/">
          <MetricCard
            label={t('conflicts')}
            value={`${report.conflicts.filter((conflict) => !conflict.ignored).length}`}
            detail={t('active')}
          />
        </Link>
        <Link href="/free-time/">
          <MetricCard label={t('focus')} value={minutesToHours(report.focus_metrics.focus_minutes, language)} detail={t('deepWork')} />
        </Link>
        <Link href="/focus/">
          <MetricCard label={t('overloadedDays')} value={`${report.overloaded_days.length}`} detail={t('inRange')} />
        </Link>
      </section>

      <section className="content-grid">
        <div className="panel">
          <h2>{t('weeklyOverview')}</h2>
          <div className="day-list">
            {report.overloaded_days.slice(0, 7).map((day) => (
              <article key={day.date} className="row-card">
                <div>
                  <strong>{day.date}</strong>
                  <p>
                    {minutesToHours(day.meeting_minutes, language)} {t('meetings')} · {day.meeting_count} {t('events')}
                  </p>
                </div>
                <StatusPill tone={day.status.includes('Severe') ? 'bad' : 'warn'}>{translateTerm(day.status, language)}</StatusPill>
              </article>
            ))}
            {report.overloaded_days.length === 0 ? <p className="muted">{t('noOverloadedDays')}</p> : null}
          </div>
        </div>

        <div className="panel">
          <h2>{t('topRisks')}</h2>
          <ol className="risk-list">
            {report.score.negative_reasons.slice(0, 5).map((reason) => (
              <li key={reason}>{translateReportText(reason, language)}</li>
            ))}
          </ol>
          {report.score.negative_reasons.length === 0 ? <p className="muted">{t('noMajorRisks')}</p> : null}

          <h2>{t('suggestions')}</h2>
          <div className="suggestions">
            {report.suggestions.slice(0, 5).map((suggestion) => (
              <article key={suggestion.id} className="row-card">
                <div>
                  <strong>{translateReportText(suggestion.text, language)}</strong>
                  <p>{translateReportText(suggestion.reason, language)}</p>
                </div>
                <button className="button compact" type="button" onClick={() => navigator.clipboard?.writeText(suggestion.text)}>
                  {t('copy')}
                </button>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="panel">
        <h2>{t('nextFocusBlocks')}</h2>
        <div className="free-grid">
          {report.free_blocks
            .filter((block) => block.block_type === 'DeepWork')
            .slice(0, 4)
            .map((block) => (
              <article key={block.id} className="row-card">
                <strong>{formatTimeRange(block.starts_at, block.ends_at, language)}</strong>
                <span>{minutesToHours(block.duration_minutes, language)}</span>
              </article>
            ))}
        </div>
      </section>
    </div>
  );
}
