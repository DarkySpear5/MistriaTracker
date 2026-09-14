import { fireEvent, render, screen } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { LoadingGate } from './LoadingGate';

it('blocks navigation clicks while the tracker is preparing imported discoveries', () => {
  const onNavigate = vi.fn();
  render(
    <LoadingGate active message="Preparing your discoveries…">
      <button onClick={onNavigate} type="button">Villagers</button>
    </LoadingGate>,
  );

  expect(screen.getByRole('status')).toHaveTextContent('Preparing your discoveries…');
  fireEvent.click(screen.getByRole('button', { name: 'Villagers' }));
  expect(onNavigate).not.toHaveBeenCalled();
});
