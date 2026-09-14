import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { SettingsPage } from './SettingsPage';

describe('SettingsPage', () => {
  afterEach(cleanup);

  it('keeps hints reserved until spoiler-free mode is active', () => {
    const onChange = vi.fn();
    render(<SettingsPage spoilerMode="all" hintsEnabled={false} onChange={onChange} />);
    expect(screen.getByRole('checkbox', { name: 'Enable gentle hints' })).toBeDisabled();
    fireEvent.click(screen.getByRole('radio', { name: 'Spoiler-free' }));
    expect(onChange).toHaveBeenCalledWith({ spoilerMode: 'free', hintsEnabled: false });
  });

  it('offers English and French without changing spoiler settings', () => {
    const onLanguageChange = vi.fn();
    render(
      <SettingsPage
        hintsEnabled={false}
        language="eng"
        onChange={vi.fn()}
        onLanguageChange={onLanguageChange}
        spoilerMode="free"
      />,
    );

    fireEvent.click(screen.getByRole('radio', { name: 'Français' }));
    expect(onLanguageChange).toHaveBeenCalledWith('fra');
  });

  it('renders safety settings in French when French is selected', () => {
    render(<SettingsPage hintsEnabled={false} language="fra" onChange={vi.fn()} spoilerMode="free" />);

    expect(screen.getByRole('heading', { name: 'Réglages' })).toBeVisible();
    expect(screen.getByRole('radio', { name: 'Sans spoiler' })).toBeChecked();
  });

  it('shows automatic live-tracking status without a user control', () => {
    render(
      <SettingsPage
        hintsEnabled={false}
        liveReadiness={{ state: 'idle' }}
        onChange={vi.fn()}
        spoilerMode="free"
      />,
    );

    expect(screen.getByRole('heading', { name: 'Live tracking safety' })).toBeVisible();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('offers an explicit backup-only import for existing discoveries', () => {
    const onImportExisting = vi.fn();
    render(
      <SettingsPage
        hintsEnabled={false}
        onChange={vi.fn()}
        onImportExisting={onImportExisting}
        spoilerMode="free"
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Import existing discoveries' }));
    expect(onImportExisting).toHaveBeenCalledOnce();
  });
});
