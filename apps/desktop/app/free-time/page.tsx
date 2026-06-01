'use client';

import { useEffect, useState } from 'react';
import { analyzeCalendar, getSettings, updateSettings } from '../../lib/tauri';
import type { AnalysisReportDto, SettingsDto } from '../../lib/types';
import { formatTimeRange, minutesToHours } from '../../lib/format-date';
import { StatusPill } from '../../components/common/StatusPill';
import { localeForLanguage, translateTerm, useI18n } from '../../lib/i18n';

export default function FreeTimePage() {
  const { language, t } = useI18n();
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
          <h1>{t('freeTime')}</h1>
          <p>{report?.free_blocks.length ?? 0} {t('blockCount')}</p>
        </div>
        {settings ? (
          <label className="inline-control">
            {t('minimumDeepWork')}
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
                <strong>{new Date(block.starts_at).toLocaleDateString(localeForLanguage(language))}</strong>
                <p>{formatTimeRange(block.starts_at, block.ends_at, language)}</p>
              </div>
              <StatusPill tone={block.block_type === 'DeepWork' ? 'ok' : block.block_type === 'MicroGap' ? 'bad' : 'warn'}>
                {translateTerm(block.block_type, language)}
              </StatusPill>
              <span>{minutesToHours(block.duration_minutes, language)}</span>
              <button className="button compact" type="button" onClick={() => navigator.clipboard?.writeText(`${block.starts_at} - ${block.ends_at}`)}>
                {t('copy')}
              </button>
            </article>
          ))}
          {report?.free_blocks.length === 0 ? <p className="muted">{t('noFreeBlocks')}</p> : null}
        </div>
      </section>
    </div>
  );
}
