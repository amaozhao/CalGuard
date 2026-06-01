'use client';

import { useEffect, useState } from 'react';
import { markdownPreview, privacySummary } from '../../lib/report';
import { analyzeCalendar, chooseReportPath, exportReport, getSettings } from '../../lib/tauri';
import type { AnalysisReportDto, ExportPrivacyDto, SettingsDto } from '../../lib/types';
import { useI18n } from '../../lib/i18n';

export default function ReportsPage() {
  const { language, t } = useI18n();
  const [settings, setSettings] = useState<SettingsDto | null>(null);
  const [report, setReport] = useState<AnalysisReportDto | null>(null);
  const [format, setFormat] = useState<'markdown' | 'json'>('markdown');
  const [privacy, setPrivacy] = useState<ExportPrivacyDto>({
    include_event_titles: true,
    include_locations: false,
    include_descriptions: false,
    include_source_names: true,
  });
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      const nextSettings = await getSettings();
      setSettings(nextSettings);
      setReport(await analyzeCalendar(nextSettings.range_days, [], nextSettings.timezone));
    }
    void load();
  }, []);

  async function saveReport() {
    if (!settings) return;
    const filePath = await chooseReportPath(format);
    if (!filePath) return;
    const result = await exportReport({
      file_path: filePath,
      format,
      range_days: settings.range_days,
      privacy,
    });
    setMessage(language === 'zh' ? `已保存 ${result.bytes_written} 字节到 ${result.file_path}` : `Saved ${result.bytes_written} bytes to ${result.file_path}`);
  }

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <h1>{t('reports')}</h1>
          <p>{privacySummary(privacy, language)}</p>
        </div>
        <div className="toolbar">
          <button className={format === 'markdown' ? 'segmented active' : 'segmented'} type="button" onClick={() => setFormat('markdown')}>
            {t('markdown')}
          </button>
          <button className={format === 'json' ? 'segmented active' : 'segmented'} type="button" onClick={() => setFormat('json')}>
            {t('json')}
          </button>
        </div>
      </header>

      <section className="content-grid">
        <div className="panel form-panel">
          <h2>{t('privacy')}</h2>
          {[
            ['include_event_titles', t('includeEventTitles')],
            ['include_locations', t('includeLocations')],
            ['include_descriptions', t('includeDescriptions')],
            ['include_source_names', t('includeSourceNames')],
          ].map(([key, label]) => (
            <label key={key} className="toggle-label">
              <input
                type="checkbox"
                checked={Boolean(privacy[key as keyof ExportPrivacyDto])}
                onChange={(event) => setPrivacy({ ...privacy, [key]: event.target.checked })}
              />
              {label}
            </label>
          ))}
          <div className="notice">{t('reviewSensitiveDetails')}</div>
          <button className="button primary" type="button" onClick={() => void saveReport()}>
            {t('saveReport')}
          </button>
          {message ? <p className="success-text">{message}</p> : null}
        </div>

        <div className="panel">
          <h2>{t('preview')}</h2>
          <pre className="preview">{report ? markdownPreview(report, privacy, language) : t('loadingReport')}</pre>
          <button
            className="button"
            type="button"
            onClick={() => report && navigator.clipboard?.writeText(markdownPreview(report, privacy, language))}
          >
            {t('copyPreview')}
          </button>
        </div>
      </section>
    </div>
  );
}
