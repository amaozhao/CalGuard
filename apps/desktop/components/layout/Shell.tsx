import Link from 'next/link';

const navItems = [
  ['Dashboard', '/dashboard/'],
  ['Sources', '/sources/'],
  ['Conflicts', '/conflicts/'],
  ['Free Time', '/free-time/'],
  ['Focus', '/focus/'],
  ['Reports', '/reports/'],
  ['Settings', '/settings/'],
] as const;

export function Shell({ children }: { children: React.ReactNode }) {
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
              {label}
            </Link>
          ))}
        </nav>
      </aside>
      <main className="main">{children}</main>
    </div>
  );
}
