'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { CalendarSourceFields } from '../../components/sources/CalendarSourceFields';
import { getSettings, updateSettings } from '../../lib/tauri';
import { addCalendarSourceFromDraft, initialSourceDraft } from '../../lib/source-form';
import { formatDayCount, translateReportText, useI18n } from '../../lib/i18n';

export default function OnboardingPage() {
  const { language, t } = useI18n();
  const router = useRouter();
  const [name, setName] = useState(initialSourceDraft.name);
  const [mode, setMode] = useState(initialSourceDraft.mode);
  const [path, setPath] = useState(initialSourceDraft.path);
  const [url, setUrl] = useState(initialSourceDraft.url);
  const [rangeDays, setRangeDays] = useState(14);
  const [workStart, setWorkStart] = useState('09:00');
  const [workEnd, setWorkEnd] = useState('18:00');
  const [minFocus, setMinFocus] = useState(90);
  const [error, setError] = useState<string | null>(null);

  async function submit() {
    setError(null);
    try {
      const settings = await getSettings();
      await updateSettings({
        ...settings,
        range_days: rangeDays,
        work_start: workStart,
        work_end: workEnd,
        min_focus_minutes: minFocus,
      });
      await addCalendarSourceFromDraft({ name, mode, path, url });
      router.push('/dashboard/');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Import failed');
    }
  }

  return (
    <div className="page-stack narrow">
      <header className="page-header">
        <div>
          <h1>CalGuard</h1>
          <p>{t('yourCalendarDataDevice')}</p>
        </div>
      </header>

      <section className="panel form-panel">
        <CalendarSourceFields
          name={name}
          setName={setName}
          mode={mode}
          setMode={setMode}
          path={path}
          setPath={setPath}
          url={url}
          setUrl={setUrl}
          modeControl="segmented"
          nameLabel={t('calendarName')}
          localLabel={t('localFilePath')}
          remoteLabel={t('remoteIcsUrl')}
          localPlaceholder="/path/to/calendar.ics"
          remotePlaceholder="https://example.com/calendar.ics"
        />

        <div className="form-grid">
          <label>
            {t('range')}
            <select value={rangeDays} onChange={(event) => setRangeDays(Number(event.target.value))}>
              <option value={7}>{formatDayCount(7, language)}</option>
              <option value={14}>{formatDayCount(14, language)}</option>
              <option value={30}>{formatDayCount(30, language)}</option>
            </select>
          </label>
          <label>
            {t('workStart')}
            <input type="time" value={workStart} onChange={(event) => setWorkStart(event.target.value)} />
          </label>
          <label>
            {t('workEnd')}
            <input type="time" value={workEnd} onChange={(event) => setWorkEnd(event.target.value)} />
          </label>
          <label>
            {t('focusTarget')}
            <input type="number" min={30} max={240} value={minFocus} onChange={(event) => setMinFocus(Number(event.target.value))} />
          </label>
        </div>

        {error ? <div className="error-banner">{translateReportText(error, language)}</div> : null}

        <button className="button primary" type="button" onClick={() => void submit()}>
          {t('generateAnalysis')}
        </button>
      </section>
    </div>
  );
}
