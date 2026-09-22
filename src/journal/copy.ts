import { translate, type SupportedLocale } from "../i18n";

export const tr = (
  language: SupportedLocale,
  key: string,
  variables?: Record<string, string | number>,
) => translate(language, key, variables);

export const categoryOrder = [
  "fish",
  "bugs",
  "crops",
  "forageables",
  "artifacts",
  "dishes",
  "recipes",
  "furniture",
  "blacksmithing",
  "materials",
  "ranching",
  "animals",
  "cosmetics",
  "perks",
  "scrolls",
  "invitations",
  "dating",
  "songs",
  "other",
];
