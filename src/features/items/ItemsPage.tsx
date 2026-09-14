import { useEffect, useState } from 'react';
import { NoteEditor } from '../notes/NoteEditor';

type Item = { id: string; name: string; description: string; icon_sprite?: string | null; seasons: string[]; locations: string[] };
type Language = 'eng' | 'fra';

const copy = {
  eng: { eyebrow: 'ITEMS', title: 'Item tracker', sort: 'Sort visible items', name: 'Name', season: 'Season', place: 'Place', previous: 'Previous items', next: 'More items' },
  fra: { eyebrow: 'OBJETS', title: 'Suivi des objets', sort: 'Trier les objets visibles', name: 'Nom', season: 'Saison', place: 'Lieu', previous: 'Objets précédents', next: 'Plus d’objets' },
} as const;

const ITEMS_PER_PAGE = 12;
const ICON_CONCURRENCY = 2;

type Props = {
  items: Item[];
  language?: Language;
  loadIcon?: (itemId: string) => Promise<string | null>;
  notes?: Record<string, string>;
  onSaveNote?: (itemId: string, text: string) => void;
  onSelectItem?: (itemId: string) => void;
  query?: string;
};

export function ItemsPage({ items, language = 'eng', loadIcon, notes = {}, onSaveNote, onSelectItem, query = '' }: Props) {
  const text = copy[language];
  const [season, setSeason] = useState<string>();
  const [location, setLocation] = useState<string>();
  const [selectedItemId, setSelectedItemId] = useState<string>();
  const [sortBy, setSortBy] = useState<'name' | 'season' | 'place'>('name');
  const [icons, setIcons] = useState<Record<string, string>>({});
  const [page, setPage] = useState(0);

  const seasons = [...new Set(items.flatMap((item) => item.seasons))].sort();
  const locations = [...new Set(items.flatMap((item) => item.locations))].sort();
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const visible = items.filter((item) =>
    (!season || item.seasons.includes(season))
    && (!location || item.locations.includes(location))
    && (!normalizedQuery || item.name.toLocaleLowerCase().includes(normalizedQuery)),
  );
  const sorted = [...visible].sort((left, right) => {
    const leftKey = sortBy === 'name' ? left.name : sortBy === 'season' ? (left.seasons[0] ?? '') : (left.locations[0] ?? '');
    const rightKey = sortBy === 'name' ? right.name : sortBy === 'season' ? (right.seasons[0] ?? '') : (right.locations[0] ?? '');
    return leftKey.localeCompare(rightKey) || left.name.localeCompare(right.name);
  });
  const totalPages = Math.max(1, Math.ceil(sorted.length / ITEMS_PER_PAGE));
  const currentPage = Math.min(page, totalPages - 1);
  const pageItems = sorted.slice(currentPage * ITEMS_PER_PAGE, (currentPage + 1) * ITEMS_PER_PAGE);

  useEffect(() => {
    if (!loadIcon) return;
    let active = true;
    const pending = pageItems.filter((item) => !icons[item.id]);
    void (async () => {
      const loaded: Record<string, string> = {};
      let next = 0;
      const worker = async () => {
        while (next < pending.length) {
          const item = pending[next++];
          const icon = await loadIcon(item.id).catch(() => null);
          if (icon) loaded[item.id] = icon;
        }
      };
      await Promise.all(Array.from({ length: Math.min(ICON_CONCURRENCY, pending.length) }, worker));
      if (active && Object.keys(loaded).length) setIcons((current) => ({ ...current, ...loaded }));
    })();
    return () => { active = false; };
  }, [icons, loadIcon, pageItems]);

  const selectedItem = sorted.find((item) => item.id === selectedItemId);

  return <section aria-labelledby="items-title">
    <p className="eyebrow">{text.eyebrow}</p>
    <h2 id="items-title">{text.title}</h2>
    <div className="filter-row">
      {seasons.map((value) => <button key={value} aria-pressed={season === value} onClick={() => setSeason(season === value ? undefined : value)}>{value}</button>)}
      {locations.map((value) => <button key={value} aria-pressed={location === value} onClick={() => setLocation(location === value ? undefined : value)}>{value}</button>)}
    </div>
    <div className="filter-row" aria-label={text.sort}>
      <button aria-pressed={sortBy === 'name'} onClick={() => setSortBy('name')} type="button">{text.name}</button>
      <button aria-pressed={sortBy === 'season'} onClick={() => setSortBy('season')} type="button">{text.season}</button>
      <button aria-pressed={sortBy === 'place'} onClick={() => setSortBy('place')} type="button">{text.place}</button>
    </div>
    <div className="collection-grid">
      {pageItems.map((item) => <article className="collection-card" key={item.id}>
        <button aria-expanded={selectedItem?.id === item.id} onClick={() => { setSelectedItemId(item.id); onSelectItem?.(item.id); }} type="button">
          {icons[item.id] ? <img alt="" className="item-icon" src={icons[item.id]} /> : <span aria-hidden="true" className="item-icon item-icon-placeholder" />}
          {item.name}
        </button>
      </article>)}
    </div>
    {totalPages > 1 && <div className="filter-row" aria-label="Item pages">
      <button disabled={currentPage === 0} onClick={() => setPage(currentPage - 1)} type="button">{text.previous}</button>
      <button disabled={currentPage + 1 === totalPages} onClick={() => setPage(currentPage + 1)} type="button">{text.next}</button>
    </div>}
    {selectedItem && <article className="item-detail">
      {icons[selectedItem.id] && <img alt="" className="item-detail-icon" src={icons[selectedItem.id]} />}
      <h3>{selectedItem.name}</h3>
      <p>{selectedItem.description}</p>
      {onSaveNote && <NoteEditor key={selectedItem.id} initialValue={notes[selectedItem.id] ?? ''} onSave={(text) => onSaveNote(selectedItem.id, text)} />}
    </article>}
  </section>;
}
