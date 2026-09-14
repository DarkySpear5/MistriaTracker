import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CollectionsPage } from './CollectionsPage';

describe('CollectionsPage', () => {
  it('shows totals and generic locks without leaking hidden identity', () => {
    render(<CollectionsPage completed={1} total={2} items={[{ id: 'paper_pondshell', name: 'Paper Pondshell' }]} />);
    expect(screen.getByText('1 / 2')).toBeVisible();
    expect(screen.getAllByLabelText('Undiscovered entry')).toHaveLength(1);
    expect(screen.queryByText('Secret Fish')).not.toBeInTheDocument();
  });

  it('keeps hidden entries generic in French', () => {
    render(<CollectionsPage completed={1} language="fra" total={2} items={[{ id: 'paper_pondshell', name: 'Coquille de papier' }]} />);

    expect(screen.getByRole('heading', { name: 'Vos découvertes' })).toBeVisible();
    expect(screen.getAllByLabelText('Entrée non découverte')).toHaveLength(1);
    expect(screen.queryByText('Poisson secret')).not.toBeInTheDocument();
  });
});
