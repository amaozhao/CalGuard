'use client';

import { useEffect, useState } from 'react';
import {
  listCalendarSources,
  removeCalendarSource,
  syncCalendarSource,
  updateCalendarSource,
} from '../../lib/tauri';
import type { CalendarSourceDto } from '../../lib/types';
import { StatusPill } from '../../components/common/StatusPill';
import { CalendarSourceFields } from '../../components/sources/CalendarSourceFields';
import { addCalendarSourceFromDraft, initialSourceDraft } from '../../lib/source-form';
import { localeForLanguage, translateReportText, translateTerm, useI18n } from '../../lib/i18n';

export default function SourcesPage() {
  const { language, t } = useI18n();
  const [sources, setSources] = useState<CalendarSourceDto[]>([]);
  const [name, setName] = useState(initialSourceDraft.name);
  const [path, setPath] = useState(initialSourceDraft.path);
  const [url, setUrl] = useState(initialSourceDraft.url);
  const [mode, setMode] = useState(initialSourceDraft.mode);
  const [error, setError] = useState<string | null>(null);

  async function load() {
    setSources(await listCalendarSources());
  }

  useEffect(() => {
    void load();
  }, []);

  async function addSource() {
    setError(null);
    try {
      await addCalendarSourceFromDraft({ name, mode, path, url }, { requireLocalIcsExtension: true });
      setPath(initialSourceDraft.path);
      setUrl(initialSourceDraft.url);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add source');
    }
  }

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <h1>{t('calendarSources')}</h1>
          <p>{sources.length} {t('sourceCount')}</p>
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
          modeControl="select"
          nameLabel={t('name')}
          localLabel={t('file')}
          remoteLabel={t('url')}
        />
        {error ? <div className="error-banner">{translateReportText(error, language)}</div> : null}
        <button className="button primary" type="button" onClick={() => void addSource()}>
          {t('addSource')}
        </button>
      </section>

      <section className="panel">
        <div className="source-list">
          {sources.map((source) => (
            <article key={source.id} className="source-row">
              <span className="swatch" style={{ backgroundColor: source.color ?? '#64748b' }} />
              <div>
                <strong>{source.name}</strong>
                <p>
                  {source.kind === 'remote_ics_url' ? t('remoteIcs') : t('localFile')} ·{' '}
                  {source.last_synced_at ? new Date(source.last_synced_at).toLocaleString(localeForLanguage(language)) : t('notSynced')}
                </p>
                {source.error_message ? <p className="error-text">{source.error_message}</p> : null}
              </div>
              <StatusPill tone={source.sync_status === 'failed' ? 'bad' : source.enabled ? 'ok' : 'idle'}>
                {translateTerm(source.enabled ? source.sync_status : 'disabled', language)}
              </StatusPill>
              <label className="toggle-label">
                <input
                  type="checkbox"
                  checked={source.enabled}
                  onChange={async (event) => {
                    await updateCalendarSource({ source_id: source.id, enabled: event.target.checked });
                    await load();
                  }}
                />
                {t('enabled')}
              </label>
              <button className="button compact" type="button" onClick={async () => {
                await syncCalendarSource(source.id);
                await load();
              }}>
                {t('refresh')}
              </button>
              <button className="button danger compact" type="button" onClick={async () => {
                await removeCalendarSource(source.id);
                await load();
              }}>
                {t('delete')}
              </button>
            </article>
          ))}
        </div>
      </section>
    </div>
  );
}
