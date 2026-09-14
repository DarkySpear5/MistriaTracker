import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Entry, Language, Source } from "./types";
import { Artwork } from "./Artwork";
import { Icon } from "./Icon";
import { tr } from "./copy";
export function DetailPanel({
  entry,
  language,
  onClose,
  onSource,
  onEntry,
  native,
  onDirtyChange,
  profileId,
  draft,
  onDraftChange,
}: {
  entry: Entry;
  language: Language;
  onClose: () => void;
  onSource: (source: Source) => void;
  onEntry: (key: string) => void;
  native: boolean;
  onDirtyChange?: (dirty: boolean) => void;
  profileId: string;
  draft?: string;
  onDraftChange?: (draft: string | undefined) => void;
}) {
  const [note, setNote] = useState("");
  const [saved, setSaved] = useState("");
  const [status, setStatus] = useState("");
  const [ready, setReady] = useState(!native);
  const [saving, setSaving] = useState(false);
  const closeRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    closeRef.current?.focus();
    let current = true;
    setNote(draft ?? "");
    setSaved("");
    setStatus("");
    setReady(!native);
    if (native)
      void invoke<string | null>("read_journal_note", {
        key: entry.key,
        profileId,
      })
        .then((value) => {
          if (current) {
            setNote(draft ?? value ?? "");
            setSaved(value ?? "");
            setReady(true);
          }
        })
        .catch(() => current && setStatus("noteError"));
    return () => {
      current = false;
    };
  }, [entry.key, profileId, native]);
  useEffect(() => {
    onDirtyChange?.(note !== saved);
  }, [note, saved, onDirtyChange]);
  useEffect(() => () => onDirtyChange?.(false), [onDirtyChange]);
  const close = () => {
    if (
      note !== saved &&
      !window.confirm(
        language === "fra"
          ? "Fermer sans enregistrer la note ?"
          : "Close without saving this note?",
      )
    )
      return;
    onClose();
  };
  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        close();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  });
  const save = async () => {
    if (!native || !ready || saving) return;
    setSaving(true);
    setStatus("");
    try {
      await invoke("save_journal_note", {
        key: entry.key,
        text: note,
        profileId,
      });
      setSaved(note);
      onDraftChange?.(undefined);
      setStatus("saved");
    } catch {
      setStatus("noteError");
    } finally {
      setSaving(false);
    }
  };
  return (
    <aside className="detail-panel" aria-labelledby="detail-title">
      <header>
        <span>{tr(language, "details")}</span>
        <button
          ref={closeRef}
          className="icon-button"
          aria-label={tr(language, "close")}
          onClick={close}
        >
          <Icon name="close" />
        </button>
      </header>
      <div className="detail-scroll">
        <div className="detail-art">
          <Artwork token={entry.art} kind={entry.category} large />
        </div>
        <span className="eyebrow">{tr(language, entry.category)}</span>
        <h2 id="detail-title">{entry.name}</h2>
        <span className={`status-pill ${entry.found ? "positive" : ""}`}>
          <Icon name={entry.found ? "check" : "eye"} size={13} />
          {tr(language, entry.found ? "found" : "missing")}
        </span>
        <p className="description">{entry.description}</p>
        {entry.seasons.length > 0 && (
          <div className="detail-meta">
            <h3>{tr(language, "season")}</h3>
            <div className="chips">
              {entry.seasons.map((season) => (
                <span key={season}>{tr(language, season)}</span>
              ))}
            </div>
          </div>
        )}
        {entry.places.length > 0 && (
          <div className="detail-meta">
            <h3>{tr(language, "place")}</h3>
            <div className="chips">
              {entry.places.map((place) => (
                <span key={place}>{tr(language, place)}</span>
              ))}
            </div>
          </div>
        )}
        {entry.ingredients.length > 0 && (
          <section className="detail-section">
            <h3>{tr(language, "ingredients")}</h3>
            {entry.ingredients.map((ingredient, index) => (
              <button
                className="ingredient-row"
                key={`${ingredient.key}-${index}`}
                disabled={!ingredient.key}
                onClick={() => ingredient.key && onEntry(ingredient.key)}
              >
                <span>{ingredient.name || tr(language, "unknownItem")}</span>
                <strong>×{ingredient.count}</strong>
              </button>
            ))}
          </section>
        )}
        <section className="detail-section">
          <h3>{tr(language, "sources")}</h3>
          {entry.sources.map((source, index) => (
            <button
              className="source-link"
              key={`${source.view}-${source.key}-${index}`}
              onClick={() => onSource(source)}
            >
              <Icon name={source.view} size={16} />
              <span>
                <small>{tr(language, source.view)}</small>
                {source.view === "encyclopedia"
                  ? tr(language, source.label)
                  : source.label}
              </span>
              <Icon name="chevron" size={14} />
            </button>
          ))}
        </section>
        <section className="detail-section">
          <h3>
            <Icon name="note" size={15} />
            {tr(language, "notes")}
          </h3>
          <textarea
            disabled={!ready || saving}
            aria-label={tr(language, "notes")}
            placeholder={tr(language, "notesHelp")}
            value={note}
            maxLength={20000}
            onChange={(event) => {
              setNote(event.target.value);
              onDraftChange?.(event.target.value);
            }}
          />
          <button
            className="primary-button"
            disabled={!native || !ready || saving || note === saved}
            onClick={() => void save()}
          >
            {tr(language, "save")}
          </button>
          {status && (
            <p
              className={status === "noteError" ? "error-text" : "saved-text"}
              role="status"
            >
              {tr(language, status)}
            </p>
          )}
        </section>
      </div>
    </aside>
  );
}
