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
import { LanguageProvider } from '../lib/i18n';

vi.mock('next/navigation', () => ({
  useRouter: () => ({ push: vi.fn() }),
}));

describe('desktop UI', () => {
  beforeEach(() => {
    resetBrowserState();
    window.localStorage.clear();
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
    expect(screen.getAllByText('72')).toHaveLength(2);
    expect(screen.getByLabelText('Calendar health chart')).toBeInTheDocument();
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

  it('switches settings page copy to Chinese', async () => {
    render(
      <LanguageProvider>
        <SettingsPage />
      </LanguageProvider>,
    );

    await userEvent.selectOptions(await screen.findByLabelText('Language'), 'zh');

    expect(await screen.findByRole('heading', { name: '设置' })).toBeInTheDocument();
    expect(screen.getByLabelText('语言')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '保存' })).toBeInTheDocument();
  });

  it('switches app content from the settings language selector', async () => {
    addBrowserSource('Work', 'local_ics_file');
    render(
      <LanguageProvider>
        <SettingsPage />
        <DashboardView />
      </LanguageProvider>,
    );

    expect(await screen.findByText('Health Score')).toBeInTheDocument();
    await userEvent.selectOptions(screen.getByLabelText('Language'), 'zh');

    expect(await screen.findByRole('heading', { name: '设置' })).toBeInTheDocument();
    expect(screen.getByText('健康评分')).toBeInTheDocument();
    expect(screen.getByText('主要风险')).toBeInTheDocument();
    expect(screen.getByText(/中级冲突/)).toBeInTheDocument();
    expect(screen.getByText('建议')).toBeInTheDocument();
  });
});
