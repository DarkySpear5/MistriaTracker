import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ProfilePicker } from './ProfilePicker';

describe('ProfilePicker', () => {
  afterEach(cleanup);

  it('selects an existing local profile without exposing game-save details', () => {
    const onSelect = vi.fn();
    render(<ProfilePicker activeProfile="1849811906" profiles={["1849811906", "249165455"]} onSelect={onSelect} />);

    fireEvent.click(screen.getByRole('radio', { name: '249165455' }));
    expect(onSelect).toHaveBeenCalledWith('249165455');
    expect(screen.queryByText(/save path/i)).not.toBeInTheDocument();
  });

  it('accepts only a numeric profile prefix entered by the user', () => {
    const onSelect = vi.fn();
    render(<ProfilePicker activeProfile={null} profiles={[]} onSelect={onSelect} />);

    fireEvent.change(screen.getByRole('textbox', { name: 'Profile prefix' }), { target: { value: '1849811906' } });
    fireEvent.click(screen.getByRole('button', { name: 'Use profile' }));
    expect(onSelect).toHaveBeenCalledWith('1849811906');
  });

  it('explains tracker-only profiles in French without showing a save path', () => {
    render(<ProfilePicker activeProfile={null} language="fra" profiles={[]} onSelect={vi.fn()} />);

    expect(screen.getByRole('heading', { name: 'Profil du suivi' })).toBeVisible();
    expect(screen.getByText(/Les profils sont enregistrés uniquement dans Mistria Tracker/)).toBeVisible();
    expect(screen.queryByText(/chemin de sauvegarde/i)).not.toBeInTheDocument();
  });
});
