'use client';

import { useEffect, useState } from 'react';
import {
  addLocalIcsSource,
  addRemoteIcsSource,
  chooseIcsFile,
  listCalendarSources,
  removeCalendarSource,
  syncCalendarSource,
  updateCalendarSource,
} from '../../lib/tauri';
import type { CalendarSourceDto } from '../../lib/types';
import { StatusPill } from '../../components/common/StatusPill';

export default function SourcesPage() {
  const [sources, setSources] = useState<CalendarSourceDto[]>([]);
  const [name, setName] = useState('My Calendar');
  const [path, setPath] = useState('');
  const [url, setUrl] = useState('');
  const [mode, setMode] = useState<'local' | 'remote'>('local');
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
      if (mode === 'local') {
        if (!path || !path.toLowerCase().endsWith('.ics')) {
          throw new Error('Select an .ics file');
        }
        await addLocalIcsSource({ name, path, color: '#2563eb' });
      } else {
        if (!url.startsWith('http://') && !url.startsWith('https://')) {
          throw new Error('Use an HTTP or HTTPS ICS URL');
        }
        await addRemoteIcsSource({ name, url, color: '#0f766e' });
      }
      setPath('');
      setUrl('');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add source');
    }
  }

  return (
    <div className="page-stack">
      <header className="page-header">
        <div>
          <h1>Calendar Sources</h1>
          <p>{sources.length} source(s)</p>
        </div>
      </header>

      <section className="panel form-panel">
        <div className="form-grid">
          <label>
            Name
            <input value={name} onChange={(event) => setName(event.target.value)} />
          </label>
          <label>
            Type
            <select value={mode} onChange={(event) => setMode(event.target.value as 'local' | 'remote')}>
              <option value="local">Local ICS</option>
              <option value="remote">Remote ICS</option>
            </select>
          </label>
        </div>

        {mode === 'local' ? (
          <label>
            File
            <div className="input-row">
              <input value={path} onChange={(event) => setPath(event.target.value)} />
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
            URL
            <input value={url} onChange={(event) => setUrl(event.target.value)} />
          </label>
        )}
        {error ? <div className="error-banner">{error}</div> : null}
        <button className="button primary" type="button" onClick={() => void addSource()}>
          Add Source
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
                  {source.kind === 'remote_ics_url' ? 'Remote ICS' : 'Local file'} ·{' '}
                  {source.last_synced_at ? new Date(source.last_synced_at).toLocaleString() : 'Not synced'}
                </p>
                {source.error_message ? <p className="error-text">{source.error_message}</p> : null}
              </div>
              <StatusPill tone={source.sync_status === 'failed' ? 'bad' : source.enabled ? 'ok' : 'idle'}>
                {source.enabled ? source.sync_status : 'disabled'}
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
                Enabled
              </label>
              <button className="button compact" type="button" onClick={async () => {
                await syncCalendarSource(source.id);
                await load();
              }}>
                Refresh
              </button>
              <button className="button danger compact" type="button" onClick={async () => {
                await removeCalendarSource(source.id);
                await load();
              }}>
                Delete
              </button>
            </article>
          ))}
        </div>
      </section>
    </div>
  );
}
