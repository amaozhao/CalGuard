import { describe, expect, it } from 'vitest';
import { filterConflicts, markdownPreview, privacySummary } from '../lib/report';
import { addBrowserSource, mockReport, resetBrowserState } from '../lib/mock-data';

describe('report helpers', () => {
  it('filters conflicts by severity', () => {
    resetBrowserState();
    addBrowserSource('Work', 'local_ics_file');
    const report = mockReport(14);

    expect(filterConflicts(report.conflicts, 'All')).toHaveLength(report.conflicts.length);
    expect(filterConflicts(report.conflicts, 'Medium')).toHaveLength(1);
    expect(filterConflicts(report.conflicts, 'High')).toHaveLength(0);
  });

  it('summarizes privacy options and redacts preview titles', () => {
    resetBrowserState();
    addBrowserSource('Work', 'local_ics_file');
    const report = mockReport(14);
    const privacy = {
      include_event_titles: false,
      include_locations: false,
      include_descriptions: false,
      include_source_names: true,
    };

    expect(privacySummary(privacy)).toContain('event titles');
    expect(markdownPreview(report, privacy)).toContain('Busy Event');
    expect(markdownPreview(report, privacy)).not.toContain('Product Review');
  });
});
