'use client';

import { useEffect, useState } from 'react';
import { filterConflicts } from '../../lib/report';
import { analyzeCalendar, getSettings, ignoreConflict } from '../../lib/tauri';
import type { AnalysisReportDto } from '../../lib/types';
import { formatTimeRange } from '../../lib/format-date';
import { StatusPill } from '../../components/common/StatusPill';
import { translateTerm, useI18n } from '../../lib/i18n';

const severities = ['All', 'Critical', 'High', 'Medium', 'Low'];

export default function ConflictsPage() {
  const { language, t } = useI18n();
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
          <h1>{t('conflicts')}</h1>
          <p>{conflicts.length} {t('activeConflicts')}</p>
        </div>
        <div className="toolbar">
          {severities.map((item) => (
            <button key={item} className={severity === item ? 'segmented active' : 'segmented'} type="button" onClick={() => setSeverity(item)}>
              {translateTerm(item, language)}
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
                  <strong>{formatTimeRange(conflict.starts_at, conflict.ends_at, language)}</strong>
                  <StatusPill tone={conflict.severity === 'Critical' || conflict.severity === 'High' ? 'bad' : 'warn'}>
                    {translateTerm(conflict.severity, language)}
                  </StatusPill>
                </div>
                <p>{conflict.event_titles.join(language === 'zh' ? ' 与 ' : ' overlaps ')}</p>
                <p>{t('overlap')}: {conflict.overlap_minutes}{language === 'zh' ? '分钟' : 'm'}</p>
              </div>
              <div className="row-actions">
                <button className="button compact" type="button" onClick={() => navigator.clipboard?.writeText(`Review ${conflict.event_titles.join(' / ')}`)}>
                  {t('copy')}
                </button>
                <button className="button compact" type="button" onClick={async () => {
                  await ignoreConflict(conflict.id);
                  await load();
                }}>
                  {t('ignore')}
                </button>
              </div>
            </article>
          ))}
          {conflicts.length === 0 ? <p className="muted">{t('noConflictsMatchFilter')}</p> : null}
        </div>
      </section>
    </div>
  );
}
