import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ItemsPage } from './ItemsPage';

describe('ItemsPage', () => {
  afterEach(cleanup);

  it('combines season and location filters without rendering hidden entries', () => {
    render(<ItemsPage items={[
      { id: 'paper_pondshell', name: 'Paper Pondshell', description: 'A folded shell.', seasons: ['Spring'], locations: ['Pond'] },
      { id: 'garden_bean', name: 'Garden Bean', description: 'A tender crop.', seasons: ['Spring'], locations: ['Farm'] },
    ]} />);
    fireEvent.click(screen.getByRole('button', { name: 'Spring' }));
    fireEvent.click(screen.getByRole('button', { name: 'Pond' }));
    expect(screen.getByText('Paper Pondshell')).toBeVisible();
    expect(screen.queryByText('Garden Bean')).not.toBeInTheDocument();
    expect(screen.queryByText('Secret Fish')).not.toBeInTheDocument();
  });

  it('searches only the visible item input it receives', () => {
    render(<ItemsPage query="pond" items={[
      { id: 'paper_pondshell', name: 'Paper Pondshell', description: 'A folded shell.', seasons: ['Spring'], locations: ['Pond'] },
      { id: 'garden_bean', name: 'Garden Bean', description: 'A tender crop.', seasons: ['Spring'], locations: ['Farm'] },
    ]} />);

    expect(screen.getByText('Paper Pondshell')).toBeVisible();
    expect(screen.queryByText('Garden Bean')).not.toBeInTheDocument();
    expect(screen.queryByText('Secret Fish')).not.toBeInTheDocument();
  });

  it('derives season and place filters from the visible items', () => {
    render(<ItemsPage items={[
      { id: 'ocean_shell', name: 'Ocean Shell', description: 'A sea shell.', seasons: ['Summer'], locations: ['Ocean'] },
      { id: 'garden_bean', name: 'Garden Bean', description: 'A tender crop.', seasons: ['Spring'], locations: ['Farm'] },
    ]} />);

    fireEvent.click(screen.getByRole('button', { name: 'Summer' }));
    fireEvent.click(screen.getByRole('button', { name: 'Ocean' }));
    expect(screen.getByRole('button', { name: 'Ocean Shell' })).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Garden Bean' })).not.toBeInTheDocument();
  });

  it('uses French tracker labels when requested', () => {
    render(<ItemsPage language="fra" items={[]} />);

    expect(screen.getByRole('heading', { name: 'Suivi des objets' })).toBeVisible();
  });

  it('uses French labels for sorting controls', () => {
    render(<ItemsPage language="fra" items={[]} />);

    expect(screen.getByRole('button', { name: 'Nom' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Saison' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Lieu' })).toBeVisible();
  });

  it('sorts visible items by place without adding hidden items', () => {
    render(<ItemsPage items={[
      { id: 'ocean_shell', name: 'Ocean Shell', description: 'A sea shell.', seasons: ['Summer'], locations: ['Ocean'] },
      { id: 'garden_bean', name: 'Garden Bean', description: 'A tender crop.', seasons: ['Spring'], locations: ['Farm'] },
    ]} />);

    fireEvent.click(screen.getByRole('button', { name: 'Place' }));
    expect(screen.getAllByRole('button', { name: /Ocean Shell|Garden Bean/ }).map((element) => element.textContent)).toEqual(['Garden Bean', 'Ocean Shell']);
    expect(screen.queryByText('Secret Fish')).not.toBeInTheDocument();
  });

  it('shows a discovered item description and saves its local note by stable item id', () => {
    const onSaveNote = vi.fn();
    render(<ItemsPage items={[{ id: 'paper_pondshell', name: 'Paper Pondshell', description: 'A folded shell.', seasons: ['Spring'], locations: ['Pond'] }]} onSaveNote={onSaveNote} />);

    fireEvent.click(screen.getByRole('button', { name: 'Paper Pondshell' }));
    expect(screen.getByText('A folded shell.')).toBeVisible();
    fireEvent.change(screen.getByRole('textbox', { name: 'Notes' }), { target: { value: 'Pond route' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save note' }));
    expect(onSaveNote).toHaveBeenCalledWith('paper_pondshell', 'Pond route');
  });

  it('renders one small item page and queues icons only for that page', async () => {
    const loadIcon = vi.fn(() => new Promise<string | null>(() => undefined));
    const items = Array.from({ length: 24 }, (_, index) => ({
      id: `item_${index + 1}`,
      name: `Item ${index + 1}`,
      description: 'Tracked item.',
      icon_sprite: `spr_ui_item_item_${index + 1}`,
      seasons: [],
      locations: [],
    }));
    render(<ItemsPage items={items} loadIcon={loadIcon} />);

    expect(screen.getAllByRole('button', { name: /^Item / })).toHaveLength(12);
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(loadIcon).toHaveBeenCalledTimes(2);
  });
});
