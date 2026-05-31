import Link from 'next/link';

export function EmptyState() {
  return (
    <section className="empty-state">
      <h2>No calendar sources</h2>
      <p>Your calendar data stays on this device by default.</p>
      <div className="actions">
        <Link className="button primary" href="/onboarding/">
          Import .ics File
        </Link>
        <Link className="button" href="/sources/">
          Add ICS URL
        </Link>
      </div>
    </section>
  );
}
