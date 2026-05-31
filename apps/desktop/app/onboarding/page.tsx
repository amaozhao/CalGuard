'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { addLocalIcsSource, addRemoteIcsSource, chooseIcsFile, getSettings, updateSettings } from '../../lib/tauri';

export default function OnboardingPage() {
  const router = useRouter();
  const [name, setName] = useState('My Calendar');
  const [mode, setMode] = useState<'local' | 'remote'>('local');
  const [path, setPath] = useState('');
  const [url, setUrl] = useState('');
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
      if (mode === 'local') {
        if (!path) {
          throw new Error('Select an .ics file');
        }
        await addLocalIcsSource({ name, path, color: '#2563eb' });
      } else {
        if (!url.startsWith('http://') && !url.startsWith('https://')) {
          throw new Error('Use an HTTP or HTTPS ICS URL');
        }
        await addRemoteIcsSource({ name, url, color: '#0f766e' });
      }
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
          <p>Your calendar data stays on your device by default.</p>
        </div>
      </header>

      <section className="panel form-panel">
        <label>
          Calendar name
          <input value={name} onChange={(event) => setName(event.target.value)} />
        </label>

        <div className="segmented-row" role="group" aria-label="Import type">
          <button className={mode === 'local' ? 'segmented active' : 'segmented'} type="button" onClick={() => setMode('local')}>
            Local file
          </button>
          <button className={mode === 'remote' ? 'segmented active' : 'segmented'} type="button" onClick={() => setMode('remote')}>
            Remote URL
          </button>
        </div>

        {mode === 'local' ? (
          <label>
            Local file path
            <div className="input-row">
              <input value={path} onChange={(event) => setPath(event.target.value)} placeholder="/path/to/calendar.ics" />
              <button
                className="button"
                type="button"
                onClick={async () => {
                  const selected = await chooseIcsFile();
                  if (selected) setPath(selected);
                }}
              >
                Browse
              </button>
            </div>
          </label>
        ) : (
          <label>
            Remote ICS URL
            <input value={url} onChange={(event) => setUrl(event.target.value)} placeholder="https://example.com/calendar.ics" />
          </label>
        )}

        <div className="form-grid">
          <label>
            Range
            <select value={rangeDays} onChange={(event) => setRangeDays(Number(event.target.value))}>
              <option value={7}>7 days</option>
              <option value={14}>14 days</option>
              <option value={30}>30 days</option>
            </select>
          </label>
          <label>
            Work start
            <input type="time" value={workStart} onChange={(event) => setWorkStart(event.target.value)} />
          </label>
          <label>
            Work end
            <input type="time" value={workEnd} onChange={(event) => setWorkEnd(event.target.value)} />
          </label>
          <label>
            Focus target
            <input type="number" min={30} max={240} value={minFocus} onChange={(event) => setMinFocus(Number(event.target.value))} />
          </label>
        </div>

        {error ? <div className="error-banner">{error}</div> : null}

        <button className="button primary" type="button" onClick={() => void submit()}>
          Generate Analysis
        </button>
      </section>
    </div>
  );
}
