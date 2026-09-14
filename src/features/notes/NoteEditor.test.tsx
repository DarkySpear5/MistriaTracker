import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { NoteEditor } from './NoteEditor';

describe('NoteEditor', () => {
  afterEach(cleanup);

  it('saves local text and reports the character budget', () => {
    const onSave = vi.fn();
    render(<NoteEditor initialValue="" onSave={onSave} />);
    fireEvent.change(screen.getByRole('textbox', { name: 'Notes' }), { target: { value: 'Pond route' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save note' }));
    expect(onSave).toHaveBeenCalledWith('Pond route');
    expect(screen.getByText('10 / 20,000')).toBeVisible();
  });

  it('updates when an asynchronously loaded local note arrives', () => {
    const { rerender } = render(<NoteEditor initialValue="" onSave={() => undefined} />);

    rerender(<NoteEditor initialValue="Pond route" onSave={() => undefined} />);
    expect(screen.getByRole('textbox', { name: 'Notes' })).toHaveValue('Pond route');
  });
});
