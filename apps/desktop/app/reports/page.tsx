'use client';

import { useEffect, useState } from 'react';
import { markdownPreview, privacySummary } from '../../lib/report';
import { analyzeCalendar, chooseReportPath, exportReport, getSettings } from '../../lib/tauri';
import type { AnalysisReportDto, ExportPrivacyDto, SettingsDto } from '../../lib/types';

export default function ReportsPage() {
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
    setMessage(`Saved ${result.bytes_written} bytes to ${result.file_path}`);
  }

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <h1>Reports</h1>
          <p>{privacySummary(privacy)}</p>
        </div>
        <div className="toolbar">
          <button className={format === 'markdown' ? 'segmented active' : 'segmented'} type="button" onClick={() => setFormat('markdown')}>
            Markdown
          </button>
          <button className={format === 'json' ? 'segmented active' : 'segmented'} type="button" onClick={() => setFormat('json')}>
            JSON
          </button>
        </div>
      </header>

      <section className="content-grid">
        <div className="panel form-panel">
          <h2>Privacy</h2>
          {[
            ['include_event_titles', 'Include event titles'],
            ['include_locations', 'Include locations'],
            ['include_descriptions', 'Include descriptions'],
            ['include_source_names', 'Include source names'],
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
          <div className="notice">Review sensitive details before saving the report.</div>
          <button className="button primary" type="button" onClick={() => void saveReport()}>
            Save Report
          </button>
          {message ? <p className="success-text">{message}</p> : null}
        </div>

        <div className="panel">
          <h2>Preview</h2>
          <pre className="preview">{report ? markdownPreview(report, privacy) : 'Loading report...'}</pre>
          <button
            className="button"
            type="button"
            onClick={() => report && navigator.clipboard?.writeText(markdownPreview(report, privacy))}
          >
            Copy Preview
          </button>
        </div>
      </section>
    </div>
  );
}
