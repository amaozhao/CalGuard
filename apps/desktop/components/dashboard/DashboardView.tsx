'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { EmptyState } from '../common/EmptyState';
import { MetricCard } from '../common/MetricCard';
import { StatusPill } from '../common/StatusPill';
import { formatTimeRange, minutesToHours } from '../../lib/format-date';
import { analyzeCalendar, getSettings, listCalendarSources, syncCalendarSource } from '../../lib/tauri';
import type { AnalysisReportDto, CalendarSourceDto, SettingsDto } from '../../lib/types';

export function DashboardView() {
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
    return <section className="panel">Loading dashboard...</section>;
  }

  if (sources.length === 0) {
    return <EmptyState />;
  }

  if (!report || !settings) {
    return <section className="panel">No analysis report available.</section>;
  }

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <h1>Dashboard</h1>
          <p>
            {new Date(report.period_start).toLocaleDateString()} to {new Date(report.period_end).toLocaleDateString()}
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
              {days}d
            </button>
          ))}
          <button className="button" type="button" onClick={() => void refreshRemote()}>
            Refresh
          </button>
          <Link className="button primary" href="/reports/">
            Export
          </Link>
        </div>
      </header>

      {error ? <div className="error-banner">{error}</div> : null}

      <section className="metrics-grid">
        <Link href="/dashboard/">
          <MetricCard label="Health Score" value={`${report.score.score}`} detail={report.score.grade} />
        </Link>
        <Link href="/conflicts/">
          <MetricCard
            label="Conflicts"
            value={`${report.conflicts.filter((conflict) => !conflict.ignored).length}`}
            detail="active"
          />
        </Link>
        <Link href="/free-time/">
          <MetricCard label="Focus" value={minutesToHours(report.focus_metrics.focus_minutes)} detail="deep work" />
        </Link>
        <Link href="/focus/">
          <MetricCard label="Overloaded Days" value={`${report.overloaded_days.length}`} detail="in range" />
        </Link>
      </section>

      <section className="content-grid">
        <div className="panel">
          <h2>Weekly Overview</h2>
          <div className="day-list">
            {report.overloaded_days.slice(0, 7).map((day) => (
              <article key={day.date} className="row-card">
                <div>
                  <strong>{day.date}</strong>
                  <p>
                    {minutesToHours(day.meeting_minutes)} meetings · {day.meeting_count} events
                  </p>
                </div>
                <StatusPill tone={day.status.includes('Severe') ? 'bad' : 'warn'}>{day.status}</StatusPill>
              </article>
            ))}
            {report.overloaded_days.length === 0 ? <p className="muted">No overloaded days in range.</p> : null}
          </div>
        </div>

        <div className="panel">
          <h2>Top Risks</h2>
          <ol className="risk-list">
            {report.score.negative_reasons.slice(0, 5).map((reason) => (
              <li key={reason}>{reason}</li>
            ))}
          </ol>
          {report.score.negative_reasons.length === 0 ? <p className="muted">No major risks detected.</p> : null}

          <h2>Suggestions</h2>
          <div className="suggestions">
            {report.suggestions.slice(0, 5).map((suggestion) => (
              <article key={suggestion.id} className="row-card">
                <div>
                  <strong>{suggestion.text}</strong>
                  <p>{suggestion.reason}</p>
                </div>
                <button className="button compact" type="button" onClick={() => navigator.clipboard?.writeText(suggestion.text)}>
                  Copy
                </button>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="panel">
        <h2>Next Focus Blocks</h2>
        <div className="free-grid">
          {report.free_blocks
            .filter((block) => block.block_type === 'DeepWork')
            .slice(0, 4)
            .map((block) => (
              <article key={block.id} className="row-card">
                <strong>{formatTimeRange(block.starts_at, block.ends_at)}</strong>
                <span>{minutesToHours(block.duration_minutes)}</span>
              </article>
            ))}
        </div>
      </section>
    </div>
  );
}
