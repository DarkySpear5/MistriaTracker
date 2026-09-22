import type { SupportedLocale } from "../i18n";

export type Language = SupportedLocale;
export type View =
  "overview" | "museum" | "villagers" | "encyclopedia" | "settings";
export type Source = { view: View; key: string; label: string; entry?: string };
export type Hint = {
  kind: string;
  activity: string | null;
  areas: string[];
  seasons: string[];
  source: string | null;
};
export const genericHint = (kind: string): Hint => ({
  kind,
  activity: null,
  areas: [],
  seasons: [],
  source: null,
});
export type Entry = {
  key: string;
  id: string | null;
  category: string;
  name: string | null;
  description: string | null;
  art: string | null;
  found: boolean;
  revealed: boolean;
  donated: boolean;
  seasons: string[];
  places: string[];
  hint?: Hint | null;
  sources: Source[];
  ingredients: { key: string | null; name: string | null; count: number }[];
};
export type Gift = {
  key: string;
  entry: string | null;
  name: string | null;
  art: string | null;
  found: boolean;
  revealed: boolean;
  hint?: Hint | null;
};
export type Villager = {
  key: string;
  name: string | null;
  bio: string | null;
  art: string | null;
  met: boolean;
  revealed: boolean;
  loved: Gift[];
  liked: Gift[];
  untried: Gift[];
};
export type MuseumSet = {
  id: string;
  wing: string;
  name: string;
  items: string[];
  completed: number;
  total: number;
};
export type Category = { id: string; completed: number; total: number };
export type Journal = {
  entries: Entry[];
  sets: MuseumSet[];
  villagers: Villager[];
  categories: Category[];
  profile: { id?: string; name: string; farm: string };
  stats: Record<string, number>;
};
export type Filters = {
  season: string;
  place: string;
  state: "all" | "found" | "missing";
  sort: "name" | "name-desc" | "found" | "missing";
};
export const defaultFilters: Filters = {
  season: "",
  place: "",
  state: "all",
  sort: "name",
};
