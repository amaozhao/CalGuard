'use client';

import Link from 'next/link';
import { useI18n } from '../../lib/i18n';

export function EmptyState() {
  const { t } = useI18n();

  return (
    <section className="empty-state">
      <h2>{t('noCalendarSources')}</h2>
      <p>{t('yourCalendarDataDevice')}</p>
      <div className="actions">
        <Link className="button primary" href="/onboarding/">
          {t('importIcsFile')}
        </Link>
        <Link className="button" href="/sources/">
          {t('addIcsUrl')}
        </Link>
      </div>
    </section>
  );
}
