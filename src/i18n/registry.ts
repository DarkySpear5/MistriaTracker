import { en } from "./en";
import { fr } from "./fr";

export type SupportedLocale = "eng" | "fra";
export type LanguagePreference = "auto" | SupportedLocale;

const dictionaries = { eng: en, fra: fr } as const;

export function languageDictionary(locale: SupportedLocale) {
  return dictionaries[locale];
}
