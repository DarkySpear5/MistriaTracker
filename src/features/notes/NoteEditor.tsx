import { useEffect, useState } from "react";
import { tr } from "../../journal/copy";
import type { Language } from "../../journal/types";

const numberLocale: Record<Language, string> = {
  eng: "en",
  fra: "fr",
  spa: "es",
  chs: "zh-CN",
  cht: "zh-TW",
  jpn: "ja",
  kor: "ko",
  rus: "ru",
};

export function NoteEditor({
  initialValue,
  onSave,
  language = "eng",
}: {
  initialValue: string;
  onSave: (value: string) => void;
  language?: Language;
}) {
  const [value, setValue] = useState(initialValue);
  useEffect(() => setValue(initialValue), [initialValue]);
  return (
    <section aria-label={tr(language, "notesSection")}>
      <label htmlFor="note-editor">{tr(language, "noteLabel")}</label>
      <textarea
        id="note-editor"
        value={value}
        maxLength={20_000}
        onChange={(event) => setValue(event.target.value)}
      />
      <p>
        {value.length.toLocaleString(numberLocale[language])} / 20,000
      </p>
      <button type="button" onClick={() => onSave(value)}>
        {tr(language, "saveNote")}
      </button>
    </section>
  );
}
