'use client';

import { useEffect, useState } from 'react';
import { filterConflicts } from '../../lib/report';
import { analyzeCalendar, getSettings, ignoreConflict } from '../../lib/tauri';
import type { AnalysisReportDto } from '../../lib/types';
import { formatTimeRange } from '../../lib/format-date';
import { StatusPill } from '../../components/common/StatusPill';

const severities = ['All', 'Critical', 'High', 'Medium', 'Low'];

export default function ConflictsPage() {
  const [report, setReport] = useState<AnalysisReportDto | null>(null);
  const [severity, setSeverity] = useState('All');

  async function load() {
    const settings = await getSettings();
    setReport(await analyzeCalendar(settings.range_days, [], settings.timezone));
  }

  useEffect(() => {
    void load();
  }, []);

  const conflicts = report ? filterConflicts(report.conflicts, severity).filter((conflict) => !conflict.ignored) : [];

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <h1>Conflicts</h1>
          <p>{conflicts.length} active conflict(s)</p>
        </div>
        <div className="toolbar">
          {severities.map((item) => (
            <button key={item} className={severity === item ? 'segmented active' : 'segmented'} type="button" onClick={() => setSeverity(item)}>
              {item}
            </button>
          ))}
        </div>
      </header>

      <section className="panel">
        <div className="conflict-list">
          {conflicts.map((conflict) => (
            <article key={conflict.id} className="row-card">
              <div>
                <div className="row-title">
                  <strong>{formatTimeRange(conflict.starts_at, conflict.ends_at)}</strong>
                  <StatusPill tone={conflict.severity === 'Critical' || conflict.severity === 'High' ? 'bad' : 'warn'}>
                    {conflict.severity}
                  </StatusPill>
                </div>
                <p>{conflict.event_titles.join(' overlaps ')}</p>
                <p>Overlap: {conflict.overlap_minutes}m</p>
              </div>
              <div className="row-actions">
                <button className="button compact" type="button" onClick={() => navigator.clipboard?.writeText(`Review ${conflict.event_titles.join(' / ')}`)}>
                  Copy
                </button>
                <button className="button compact" type="button" onClick={async () => {
                  await ignoreConflict(conflict.id);
                  await load();
                }}>
                  Ignore
                </button>
              </div>
            </article>
          ))}
          {conflicts.length === 0 ? <p className="muted">No conflicts match the current filter.</p> : null}
        </div>
      </section>
    </div>
  );
}
