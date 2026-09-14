type VisibleItem = { id: string; name: string };
type Language = 'eng' | 'fra';

const copy = {
  eng: { eyebrow: 'COLLECTIONS', title: 'Your discoveries', undiscovered: 'Undiscovered entry' },
  fra: { eyebrow: 'COLLECTIONS', title: 'Vos découvertes', undiscovered: 'Entrée non découverte' },
} as const;

export function CollectionsPage({ completed, language = 'eng', total, items }: { completed: number; language?: Language; total: number; items: VisibleItem[] }) {
  const text = copy[language];
  const hidden = Math.max(0, total - items.length);
  return (
    <section aria-labelledby="collections-title">
      <p className="eyebrow">{text.eyebrow}</p>
      <h2 id="collections-title">{text.title}</h2>
      <strong>{completed} / {total}</strong>
      <div className="collection-grid">
        {items.map((item) => <article key={item.id} className="collection-card">{item.name}</article>)}
        {Array.from({ length: hidden }, (_, index) => <div key={index} className="collection-lock" aria-label={text.undiscovered}>?</div>)}
      </div>
    </section>
  );
}
