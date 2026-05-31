import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { DashboardView } from '../components/dashboard/DashboardView';
import OnboardingPage from '../app/onboarding/page';
import ReportsPage from '../app/reports/page';
import SettingsPage from '../app/settings/page';
import SourcesPage from '../app/sources/page';
import FreeTimePage from '../app/free-time/page';
import { addBrowserSource, resetBrowserState } from '../lib/mock-data';

vi.mock('next/navigation', () => ({
  useRouter: () => ({ push: vi.fn() }),
}));

describe('desktop UI', () => {
  beforeEach(() => {
    resetBrowserState();
  });

  it('shows dashboard empty state without calendar sources', async () => {
    render(<DashboardView />);

    expect(await screen.findByText('No calendar sources')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Import .ics File' })).toBeInTheDocument();
  });

  it('shows dashboard metrics with calendar data', async () => {
    addBrowserSource('Work', 'local_ics_file');
    render(<DashboardView />);

    expect(await screen.findByText('Health Score')).toBeInTheDocument();
    expect(screen.getByText('72')).toBeInTheDocument();
    expect(screen.getByText('Top Risks')).toBeInTheDocument();
  });

  it('validates onboarding local file input', async () => {
    render(<OnboardingPage />);

    await userEvent.click(screen.getByRole('button', { name: 'Generate Analysis' }));

    expect(await screen.findByText('Select an .ics file')).toBeInTheDocument();
  });

  it('validates sources add form input', async () => {
    render(<SourcesPage />);

    await userEvent.click(await screen.findByRole('button', { name: 'Add Source' }));

    expect(await screen.findByText('Select an .ics file')).toBeInTheDocument();
  });

  it('shows free time blocks', async () => {
    addBrowserSource('Work', 'local_ics_file');
    render(<FreeTimePage />);

    expect(await screen.findByText('DeepWork')).toBeInTheDocument();
    expect(screen.getByText('2.0h')).toBeInTheDocument();
  });

  it('toggles report privacy preview', async () => {
    addBrowserSource('Work', 'local_ics_file');
    render(<ReportsPage />);

    expect(await screen.findByText('Preview')).toBeInTheDocument();
    await waitFor(() => expect(screen.getByText(/Top item: Product Review/)).toBeInTheDocument());
    await userEvent.click(screen.getByLabelText('Include event titles'));

    expect(await screen.findByText(/Top item: Busy Event/)).toBeInTheDocument();
  });

  it('edits settings form values', async () => {
    render(<SettingsPage />);

    const focusInput = await screen.findByLabelText('Min focus');
    await userEvent.clear(focusInput);
    await userEvent.type(focusInput, '120');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(await screen.findByText('Settings saved')).toBeInTheDocument();
  });
});
