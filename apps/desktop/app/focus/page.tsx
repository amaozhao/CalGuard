'use client';

import { useEffect, useState } from 'react';
import { MetricCard } from '../../components/common/MetricCard';
import { minutesToHours } from '../../lib/format-date';
import { analyzeCalendar, getSettings } from '../../lib/tauri';
import type { AnalysisReportDto } from '../../lib/types';

export default function FocusPage() {
  const [report, setReport] = useState<AnalysisReportDto | null>(null);

  useEffect(() => {
    async function load() {
      const settings = await getSettings();
      setReport(await analyzeCalendar(settings.range_days, [], settings.timezone));
    }
    void load();
  }, []);

  if (!report) {
    return <section className="panel">Loading focus metrics...</section>;
  }

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <h1>Focus</h1>
          <p>Fragmentation {report.focus_metrics.fragmentation_score}/100</p>
        </div>
      </header>

      <section className="metrics-grid">
        <MetricCard label="Deep Work Blocks" value={`${report.focus_metrics.deep_work_blocks}`} />
        <MetricCard label="Focus Hours" value={minutesToHours(report.focus_metrics.focus_minutes)} />
        <MetricCard label="Longest Block" value={minutesToHours(report.focus_metrics.longest_free_block_minutes)} />
        <MetricCard label="Meeting Density" value={`${report.focus_metrics.meeting_density_percent}%`} />
      </section>

      <section className="panel">
        <h2>No Focus Days</h2>
        <div className="day-list">
          {report.focus_metrics.no_focus_days.map((date) => (
            <article key={date} className="row-card">
              <strong>{date}</strong>
              <span>No deep work block</span>
            </article>
          ))}
          {report.focus_metrics.no_focus_days.length === 0 ? <p className="muted">Every workday has deep work.</p> : null}
        </div>
      </section>
    </div>
  );
}
