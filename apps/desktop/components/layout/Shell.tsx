'use client';

import Link from 'next/link';
import { useI18n } from '../../lib/i18n';

const navItems = [
  ['dashboard', '/dashboard/'],
  ['sources', '/sources/'],
  ['conflicts', '/conflicts/'],
  ['freeTime', '/free-time/'],
  ['focus', '/focus/'],
  ['reports', '/reports/'],
  ['settings', '/settings/'],
] as const;

export function Shell({ children }: { children: React.ReactNode }) {
  const { t } = useI18n();

  return (
    <div className="shell">
      <aside className="sidebar">
        <Link className="brand" href="/dashboard/">
          <span className="brand-mark">CG</span>
          <span>CalGuard</span>
        </Link>
        <nav className="nav">
          {navItems.map(([label, href]) => (
            <Link key={href} href={href}>
              {t(label)}
            </Link>
          ))}
        </nav>
      </aside>
      <main className="main">{children}</main>
    </div>
  );
}
