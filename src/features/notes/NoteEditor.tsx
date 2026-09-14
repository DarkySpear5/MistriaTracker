import { useEffect, useState } from 'react';

export function NoteEditor({ initialValue, onSave }: { initialValue: string; onSave: (value: string) => void }) {
  const [value, setValue] = useState(initialValue);
  useEffect(() => setValue(initialValue), [initialValue]);
  return <section aria-label="Notes"><label htmlFor="note-editor">Notes</label><textarea id="note-editor" value={value} maxLength={20_000} onChange={(event) => setValue(event.target.value)} /><p>{value.length.toLocaleString('en-US')} / 20,000</p><button type="button" onClick={() => onSave(value)}>Save note</button></section>;
}
