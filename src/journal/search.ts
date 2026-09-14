import type { Entry, Filters, Journal, View } from "./types";
export function normalize(value: string) {
  return value
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLocaleLowerCase();
}
export function filterEntries(entries: Entry[], filters: Filters) {
  const list = entries.filter(
    (entry) =>
      (!filters.season || entry.seasons.includes(filters.season)) &&
      (!filters.place || entry.places.includes(filters.place)) &&
      (filters.state === "all" || entry.found === (filters.state === "found")),
  );
  return list.sort((a, b) => {
    const found = Number(b.found) - Number(a.found);
    if (filters.sort === "found" && found) return found;
    if (filters.sort === "missing" && found) return -found;
    // Hidden identities never influence visible alphabetical ordering.
    if (a.revealed !== b.revealed) return a.revealed ? -1 : 1;
    return (
      (a.name ?? "").localeCompare(b.name ?? "") *
        (filters.sort === "name-desc" ? -1 : 1) || a.key.localeCompare(b.key)
    );
  });
}
export type SearchResult = {
  key: string;
  name: string;
  art: string | null;
  type: "entry" | "villager";
  category: string;
  local: boolean;
  target: View;
};
export function searchJournal(
  journal: Journal,
  query: string,
  view: View,
  category: string | null,
): SearchResult[] {
  const normalized = normalize(query.trim());
  if (!normalized) return [];
  const museum = new Set(journal.sets.flatMap((set) => set.items));
  const gifts = new Set(
    journal.villagers.flatMap((npc) =>
      [...npc.liked, ...npc.loved]
        .filter((gift) => gift.revealed)
        .map((gift) => gift.entry),
    ),
  );
  const results: SearchResult[] = journal.entries
    .filter(
      (entry) =>
        entry.revealed &&
        entry.name &&
        normalize(entry.name).includes(normalized),
    )
    .map((entry) => {
      const local =
        view === "museum"
          ? museum.has(entry.key)
          : view === "villagers"
            ? gifts.has(entry.key)
            : view === "encyclopedia" &&
              (!category || entry.category === category);
      return {
        key: entry.key,
        name: entry.name!,
        art: entry.art,
        type: "entry",
        category: entry.category,
        local,
        target: local ? view : "encyclopedia",
      };
    });
  for (const npc of journal.villagers)
    if (npc.revealed && npc.name && normalize(npc.name).includes(normalized))
      results.push({
        key: npc.key,
        name: npc.name,
        art: npc.art,
        type: "villager",
        category: "villagers",
        local: view === "villagers",
        target: "villagers",
      });
  return results.sort(
    (a, b) => Number(b.local) - Number(a.local) || a.name.localeCompare(b.name),
  );
}
