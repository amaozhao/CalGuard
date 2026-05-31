'use client';

import { useEffect, useState } from 'react';
import { analyzeCalendar, getSettings, updateSettings } from '../../lib/tauri';
import type { AnalysisReportDto, SettingsDto } from '../../lib/types';
import { formatTimeRange, minutesToHours } from '../../lib/format-date';
import { StatusPill } from '../../components/common/StatusPill';

export default function FreeTimePage() {
  const [report, setReport] = useState<AnalysisReportDto | null>(null);
  const [settings, setSettings] = useState<SettingsDto | null>(null);

  async function load() {
    const nextSettings = await getSettings();
    setSettings(nextSettings);
    setReport(await analyzeCalendar(nextSettings.range_days, [], nextSettings.timezone));
  }

  useEffect(() => {
    void load();
  }, []);

  async function updateFocus(minutes: number) {
    if (!settings) return;
    await updateSettings({ ...settings, min_focus_minutes: minutes });
    await load();
  }

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <h1>Free Time</h1>
          <p>{report?.free_blocks.length ?? 0} block(s)</p>
        </div>
        {settings ? (
          <label className="inline-control">
            Minimum Deep Work
            <input
              type="number"
              min={30}
              max={240}
              value={settings.min_focus_minutes}
              onChange={(event) => void updateFocus(Number(event.target.value))}
            />
          </label>
        ) : null}
      </header>

      <section className="panel">
        <div className="free-grid">
          {report?.free_blocks.map((block) => (
            <article key={block.id} className="row-card">
              <div>
                <strong>{new Date(block.starts_at).toLocaleDateString()}</strong>
                <p>{formatTimeRange(block.starts_at, block.ends_at)}</p>
              </div>
              <StatusPill tone={block.block_type === 'DeepWork' ? 'ok' : block.block_type === 'MicroGap' ? 'bad' : 'warn'}>
                {block.block_type}
              </StatusPill>
              <span>{minutesToHours(block.duration_minutes)}</span>
              <button className="button compact" type="button" onClick={() => navigator.clipboard?.writeText(`${block.starts_at} - ${block.ends_at}`)}>
                Copy
              </button>
            </article>
          ))}
          {report?.free_blocks.length === 0 ? <p className="muted">No free blocks inside working hours.</p> : null}
        </div>
      </section>
    </div>
  );
}
