'use client';

import { useEffect, useState } from 'react';
import { clearCache, getSettings, updateSettings } from '../../lib/tauri';
import type { SettingsDto } from '../../lib/types';
import { formatDayCount, translateReportText, useI18n } from '../../lib/i18n';

export default function SettingsPage() {
  const { language, setLanguage, t } = useI18n();
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
    return <section className="panel">{t('loadingSettings')}</section>;
  }

  return (
    <div className="page-stack narrow">
      <header className="page-header">
        <div>
          <h1>{t('settings')}</h1>
          <p>{settings.timezone}</p>
        </div>
      </header>

      <section className="panel form-panel">
        <div className="form-grid">
          <label>
            {t('language')}
            <select value={language} onChange={(event) => setLanguage(event.target.value === 'zh' ? 'zh' : 'en')}>
              <option value="en">English</option>
              <option value="zh">中文</option>
            </select>
          </label>
          <label>
            {t('timezone')}
            <input value={settings.timezone} onChange={(event) => setSettings({ ...settings, timezone: event.target.value })} />
          </label>
          <label>
            {t('range')}
            <select value={settings.range_days} onChange={(event) => setSettings({ ...settings, range_days: Number(event.target.value) })}>
              <option value={7}>{formatDayCount(7, language)}</option>
              <option value={14}>{formatDayCount(14, language)}</option>
              <option value={30}>{formatDayCount(30, language)}</option>
            </select>
          </label>
          <label>
            {t('minFocus')}
            <input
              type="number"
              value={settings.min_focus_minutes}
              onChange={(event) => setSettings({ ...settings, min_focus_minutes: Number(event.target.value) })}
            />
          </label>
          <label>
            {t('workStart')}
            <input type="time" value={settings.work_start} onChange={(event) => setSettings({ ...settings, work_start: event.target.value })} />
          </label>
          <label>
            {t('workEnd')}
            <input type="time" value={settings.work_end} onChange={(event) => setSettings({ ...settings, work_end: event.target.value })} />
          </label>
          <label>
            {t('lunchStart')}
            <input type="time" value={settings.lunch_start} onChange={(event) => setSettings({ ...settings, lunch_start: event.target.value })} />
          </label>
          <label>
            {t('lunchEnd')}
            <input type="time" value={settings.lunch_end} onChange={(event) => setSettings({ ...settings, lunch_end: event.target.value })} />
          </label>
          <label>
            {t('overloadMinutes')}
            <input
              type="number"
              value={settings.overload_meeting_minutes}
              onChange={(event) => setSettings({ ...settings, overload_meeting_minutes: Number(event.target.value) })}
            />
          </label>
          <label>
            {t('severeOverload')}
            <input
              type="number"
              value={settings.severe_overload_meeting_minutes}
              onChange={(event) => setSettings({ ...settings, severe_overload_meeting_minutes: Number(event.target.value) })}
            />
          </label>
        </div>

        <div className="toolbar">
          <button className="button primary" type="button" onClick={() => void save(settings)}>
            {t('save')}
          </button>
          <button className="button danger" type="button" onClick={async () => {
            await clearCache();
            setMessage('Local cache cleared');
          }}>
            {t('clearCache')}
          </button>
        </div>
        {message ? <p className="success-text">{translateReportText(message, language)}</p> : null}
      </section>
    </div>
  );
}
