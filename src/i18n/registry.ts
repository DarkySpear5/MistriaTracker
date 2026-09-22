import { en, type LocaleDictionary } from "./en";
import { es } from "./es";
import { fr } from "./fr";
import { chs } from "./chs";
import { cht } from "./cht";
import { ja } from "./ja";
import { ko } from "./ko";
import { ru } from "./ru";

export const localeOptions = [
  { value: "eng", label: "english", steamLanguage: "english" },
  { value: "fra", label: "french", steamLanguage: "french" },
  { value: "spa", label: "spanish", steamLanguage: "spanish" },
  { value: "chs", label: "chineseSimplified", steamLanguage: "schinese" },
  { value: "cht", label: "chineseTraditional", steamLanguage: "tchinese" },
  { value: "jpn", label: "japanese", steamLanguage: "japanese" },
  { value: "kor", label: "korean", steamLanguage: "koreana" },
  { value: "rus", label: "russian", steamLanguage: "russian" },
] as const;

export type SupportedLocale = (typeof localeOptions)[number]["value"];
export type LanguagePreference = "auto" | SupportedLocale;

const dictionaries: Record<SupportedLocale, LocaleDictionary> = {
  eng: en,
  fra: fr,
  spa: es,
  chs,
  cht,
  jpn: ja,
  kor: ko,
  rus: ru,
};

export function languageDictionary(locale: SupportedLocale): LocaleDictionary {
  return dictionaries[locale];
}

export function translate(
  locale: SupportedLocale,
  key: string,
  variables: Record<string, string | number> = {},
): string {
  const english = en[key as keyof typeof en];
  const value = dictionaries[locale][key as keyof typeof en] ?? english;
  const template =
    typeof value === "string"
      ? value
      : typeof english === "string"
        ? english
        : key.replaceAll("_", " ");

  return template.replace(/\{([a-zA-Z0-9_]+)\}/g, (match, name: string) =>
    Object.hasOwn(variables, name) ? String(variables[name]) : match,
  );
}
