export function StatusPill({ tone, children }: { tone: 'ok' | 'warn' | 'bad' | 'idle'; children: React.ReactNode }) {
  return <span className={`status-pill status-${tone}`}>{children}</span>;
}
