import { describe, it, expect } from "vitest";
import { filterEntries, searchJournal } from "./search";
import { defaultFilters, type Entry, type Journal } from "./types";
const entry = (key: string, name: string | null, found = true): Entry => ({
  key,
  id: name ? key : null,
  name,
  category: "fish",
  description: name,
  art: null,
  found,
  revealed: name !== null,
  donated: false,
  seasons: ["spring"],
  places: ["pond"],
  sources: [],
  ingredients: [],
});
export const journal: Journal = {
  entries: [
    entry("trout", "Trout"),
    entry("brown", "Brown Trout"),
    entry("hidden", null, false),
    entry("ete", "Truite étoilée"),
  ],
  sets: [
    {
      id: "spring",
      wing: "fish",
      name: "Spring fish",
      items: ["trout", "brown"],
      completed: 0,
      total: 2,
    },
  ],
  villagers: [],
  categories: [{ id: "fish", completed: 3, total: 4 }],
  profile: { name: "Test", farm: "Test farm" },
  stats: {},
};
describe("journal navigation contracts", () => {
  it("matches partial contiguous letters ignoring accents and case", () => {
    expect(
      searchJournal(journal, "TrOu", "overview", null).map((v) => v.key),
    ).toEqual(["brown", "trout"]);
    expect(
      searchJournal(journal, "ETOI", "overview", null).map((v) => v.key),
    ).toEqual(["ete"]);
  });
  it("keeps museum results in Museum and routes other contexts into Encyclopaedia", () => {
    expect(
      searchJournal(journal, "trout", "museum", null).every(
        (result) => result.target === "museum",
      ),
    ).toBe(true);
    expect(
      searchJournal(journal, "trout", "overview", null).every(
        (result) => result.target === "encyclopedia",
      ),
    ).toBe(true);
  });
  it("does not search hidden identifiers", () =>
    expect(searchJournal(journal, "hidden", "encyclopedia", null)).toEqual([]));
  it("combines filters without guessing hidden availability", () => {
    expect(
      filterEntries(journal.entries, {
        ...defaultFilters,
        state: "found",
        season: "summer",
      }),
    ).toEqual([]);
    expect(
      filterEntries(journal.entries, {
        ...defaultFilters,
        state: "missing",
      }).map((v) => v.key),
    ).toEqual(["hidden"]);
  });
});
