'use client';

import { chooseIcsFile } from '../../lib/tauri';
import type { SourceMode } from '../../lib/source-form';
import { useI18n } from '../../lib/i18n';

type CalendarSourceFieldsProps = {
  name: string;
  setName: (value: string) => void;
  mode: SourceMode;
  setMode: (value: SourceMode) => void;
  path: string;
  setPath: (value: string) => void;
  url: string;
  setUrl: (value: string) => void;
  modeControl: 'segmented' | 'select';
  nameLabel: string;
  localLabel: string;
  remoteLabel: string;
  localPlaceholder?: string;
  remotePlaceholder?: string;
};

export function CalendarSourceFields({
  name,
  setName,
  mode,
  setMode,
  path,
  setPath,
  url,
  setUrl,
  modeControl,
  nameLabel,
  localLabel,
  remoteLabel,
  localPlaceholder,
  remotePlaceholder,
}: CalendarSourceFieldsProps) {
  const { t } = useI18n();

  async function browseLocalFile() {
    const selected = await chooseIcsFile();
    if (selected) setPath(selected);
  }

  return (
    <>
      {modeControl === 'select' ? (
        <div className="form-grid">
          <label>
            {nameLabel}
            <input value={name} onChange={(event) => setName(event.target.value)} />
          </label>
          <label>
            {t('type')}
            <select value={mode} onChange={(event) => setMode(event.target.value as SourceMode)}>
              <option value="local">{t('localIcs')}</option>
              <option value="remote">{t('remoteIcs')}</option>
            </select>
          </label>
        </div>
      ) : (
        <>
          <label>
            {nameLabel}
            <input value={name} onChange={(event) => setName(event.target.value)} />
          </label>
          <div className="segmented-row" role="group" aria-label={t('importType')}>
            <button className={mode === 'local' ? 'segmented active' : 'segmented'} type="button" onClick={() => setMode('local')}>
              {t('localFile')}
            </button>
            <button className={mode === 'remote' ? 'segmented active' : 'segmented'} type="button" onClick={() => setMode('remote')}>
              {t('remoteUrl')}
            </button>
          </div>
        </>
      )}

      {mode === 'local' ? (
        <label>
          {localLabel}
          <div className="input-row">
            <input value={path} onChange={(event) => setPath(event.target.value)} placeholder={localPlaceholder} />
            <button className="button" type="button" onClick={() => void browseLocalFile()}>
              {t('browse')}
            </button>
          </div>
        </label>
      ) : (
        <label>
          {remoteLabel}
          <input value={url} onChange={(event) => setUrl(event.target.value)} placeholder={remotePlaceholder} />
        </label>
      )}
    </>
  );
}
