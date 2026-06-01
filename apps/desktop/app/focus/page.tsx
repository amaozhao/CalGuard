'use client';

import { useEffect, useState } from 'react';
import { MetricCard } from '../../components/common/MetricCard';
import { minutesToHours } from '../../lib/format-date';
import { useI18n } from '../../lib/i18n';
import { analyzeCalendar, getSettings } from '../../lib/tauri';
import type { AnalysisReportDto } from '../../lib/types';

export default function FocusPage() {
  const { language, t } = useI18n();
  const [report, setReport] = useState<AnalysisReportDto | null>(null);

  useEffect(() => {
    async function load() {
      const settings = await getSettings();
      setReport(await analyzeCalendar(settings.range_days, [], settings.timezone));
    }
    void load();
  }, []);

  if (!report) {
    return <section className="panel">{t('loadingFocusMetrics')}</section>;
  }

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <h1>{t('focus')}</h1>
          <p>{t('fragmentation')} {report.focus_metrics.fragmentation_score}/100</p>
        </div>
      </header>

      <section className="metrics-grid">
        <MetricCard label={t('deepWorkBlocks')} value={`${report.focus_metrics.deep_work_blocks}`} />
        <MetricCard label={t('focusHours')} value={minutesToHours(report.focus_metrics.focus_minutes, language)} />
        <MetricCard label={t('longestBlock')} value={minutesToHours(report.focus_metrics.longest_free_block_minutes, language)} />
        <MetricCard label={t('meetingDensity')} value={`${report.focus_metrics.meeting_density_percent}%`} />
      </section>

      <section className="panel">
        <h2>{t('noFocusDays')}</h2>
        <div className="day-list">
          {report.focus_metrics.no_focus_days.map((date) => (
            <article key={date} className="row-card">
              <strong>{date}</strong>
              <span>{t('noDeepWorkBlock')}</span>
            </article>
          ))}
          {report.focus_metrics.no_focus_days.length === 0 ? <p className="muted">{t('everyWorkdayHasDeepWork')}</p> : null}
        </div>
      </section>
    </div>
  );
}
