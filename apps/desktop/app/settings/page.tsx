'use client';

import { useEffect, useState } from 'react';
import { clearCache, getSettings, updateSettings } from '../../lib/tauri';
import type { SettingsDto } from '../../lib/types';

export default function SettingsPage() {
  const [settings, setSettings] = useState<SettingsDto | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    void getSettings().then(setSettings);
  }, []);

  async function save(next: SettingsDto) {
    setSettings(next);
    await updateSettings(next);
    setMessage('Settings saved');
  }

  if (!settings) {
    return <section className="panel">Loading settings...</section>;
  }

  return (
    <div className="page-stack narrow">
      <header className="page-header">
        <div>
          <h1>Settings</h1>
          <p>{settings.timezone}</p>
        </div>
      </header>

      <section className="panel form-panel">
        <div className="form-grid">
          <label>
            Language
            <select defaultValue="en">
              <option value="en">English</option>
              <option value="zh">中文</option>
            </select>
          </label>
          <label>
            Timezone
            <input value={settings.timezone} onChange={(event) => setSettings({ ...settings, timezone: event.target.value })} />
          </label>
          <label>
            Range
            <select value={settings.range_days} onChange={(event) => setSettings({ ...settings, range_days: Number(event.target.value) })}>
              <option value={7}>7 days</option>
              <option value={14}>14 days</option>
              <option value={30}>30 days</option>
            </select>
          </label>
          <label>
            Min focus
            <input
              type="number"
              value={settings.min_focus_minutes}
              onChange={(event) => setSettings({ ...settings, min_focus_minutes: Number(event.target.value) })}
            />
          </label>
          <label>
            Work start
            <input type="time" value={settings.work_start} onChange={(event) => setSettings({ ...settings, work_start: event.target.value })} />
          </label>
          <label>
            Work end
            <input type="time" value={settings.work_end} onChange={(event) => setSettings({ ...settings, work_end: event.target.value })} />
          </label>
          <label>
            Lunch start
            <input type="time" value={settings.lunch_start} onChange={(event) => setSettings({ ...settings, lunch_start: event.target.value })} />
          </label>
          <label>
            Lunch end
            <input type="time" value={settings.lunch_end} onChange={(event) => setSettings({ ...settings, lunch_end: event.target.value })} />
          </label>
          <label>
            Overload minutes
            <input
              type="number"
              value={settings.overload_meeting_minutes}
              onChange={(event) => setSettings({ ...settings, overload_meeting_minutes: Number(event.target.value) })}
            />
          </label>
          <label>
            Severe overload
            <input
              type="number"
              value={settings.severe_overload_meeting_minutes}
              onChange={(event) => setSettings({ ...settings, severe_overload_meeting_minutes: Number(event.target.value) })}
            />
          </label>
        </div>

        <div className="toolbar">
          <button className="button primary" type="button" onClick={() => void save(settings)}>
            Save
          </button>
          <button className="button danger" type="button" onClick={async () => {
            await clearCache();
            setMessage('Local cache cleared');
          }}>
            Clear Cache
          </button>
        </div>
        {message ? <p className="success-text">{message}</p> : null}
      </section>
    </div>
  );
}
