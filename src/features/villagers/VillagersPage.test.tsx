import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { VillagersPage } from './VillagersPage';

describe('VillagersPage', () => {
  afterEach(cleanup);

  it('shows only observed gift pairs', () => {
    render(<VillagersPage villagers={[{ id: 'adeline', name: 'Adeline', gifts: [{ item: 'Paper Pondshell', reaction: 'Liked' }] }]} />);
    expect(screen.getByText('Paper Pondshell')).toBeVisible();
    expect(screen.getByText('Liked')).toBeVisible();
    expect(screen.queryByText('Secret favorite')).not.toBeInTheDocument();
  });

  it('searches only revealed villager and gift-pair text', () => {
    render(<VillagersPage query="pond" villagers={[
      { id: 'adeline', name: 'Adeline', gifts: [{ item: 'Paper Pondshell', reaction: 'Liked' }] },
      { id: 'juniper', name: 'Juniper', gifts: [{ item: 'Spring Bean', reaction: 'Neutral' }] },
    ]} />);

    expect(screen.getByText('Adeline')).toBeVisible();
    expect(screen.queryByText('Juniper')).not.toBeInTheDocument();
    expect(screen.queryByText('Secret favorite')).not.toBeInTheDocument();
  });

  it('uses French tracker labels when requested', () => {
    render(<VillagersPage language="fra" villagers={[]} />);

    expect(screen.getByRole('heading', { name: 'Villageois' })).toBeVisible();
  });
});
