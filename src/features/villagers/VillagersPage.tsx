type Villager = { id: string; name: string; gifts: { item: string; reaction: string }[] };
type Language = 'eng' | 'fra';

const copy = {
  eng: { eyebrow: 'VILLAGERS', title: 'Villagers' },
  fra: { eyebrow: 'VILLAGEOIS', title: 'Villageois' },
} as const;

export function VillagersPage({ villagers, language = 'eng', query = '' }: { villagers: Villager[]; language?: Language; query?: string }) {
  const text = copy[language];
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const visible = villagers.filter((villager) =>
    !normalizedQuery
    || villager.name.toLocaleLowerCase().includes(normalizedQuery)
    || villager.gifts.some((gift) => gift.item.toLocaleLowerCase().includes(normalizedQuery)),
  );
  return <section aria-labelledby="villagers-title"><p className="eyebrow">{text.eyebrow}</p><h2 id="villagers-title">{text.title}</h2>{visible.map((villager) => <article className="collection-card" key={villager.id}><h3>{villager.name}</h3>{villager.gifts.map((gift) => <p key={gift.item}><span>{gift.item}</span> · <strong>{gift.reaction}</strong></p>)}</article>)}</section>;
}
