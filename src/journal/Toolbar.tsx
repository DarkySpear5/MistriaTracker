import { useEffect, useMemo, useRef, useState } from "react";
import type { Filters, Journal, Language, View } from "./types";
import { defaultFilters } from "./types";
import { searchJournal, type SearchResult } from "./search";
import { tr } from "./copy";
import { Artwork } from "./Artwork";
import { Icon } from "./Icon";
export function Toolbar({
  journal,
  language,
  view,
  category,
  filters,
  onFilters,
  onResult,
}: {
  journal: Journal | null;
  language: Language;
  view: View;
  category: string | null;
  filters: Filters;
  onFilters: (f: Filters) => void;
  onResult: (r: SearchResult) => void;
}) {
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [index, setIndex] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const wrap = useRef<HTMLDivElement>(null);
  const results = useMemo(
    () => (journal ? searchJournal(journal, query, view, category) : []),
    [journal, query, view, category],
  );
  const choose = (result: SearchResult) => {
    setOpen(false);
    setQuery("");
    onResult(result);
  };
  useEffect(() => {
    const keys = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        input.current?.focus();
      }
      if (e.key === "Escape") setOpen(false);
    };
    const outside = (e: PointerEvent) => {
      if (!wrap.current?.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener("keydown", keys);
    window.addEventListener("pointerdown", outside);
    return () => {
      window.removeEventListener("keydown", keys);
      window.removeEventListener("pointerdown", outside);
    };
  }, []);
  const filtersEnabled =
    view === "museum" ||
    view === "villagers" ||
    (view === "encyclopedia" && !!category);
  const availabilityEnabled = filtersEnabled && view !== "villagers";
  const places = [
    ...new Set(
      journal?.entries
        .filter(
          (item) =>
            view !== "encyclopedia" || !category || item.category === category,
        )
        .flatMap((item) => item.places) ?? [],
    ),
  ].sort();
  return (
    <header className="toolbar">
      <div className="search-wrap" ref={wrap}>
        <Icon name="search" size={18} />
        <input
          ref={input}
          type="search"
          value={query}
          aria-label={tr(language, "search")}
          placeholder={tr(language, "search")}
          role="combobox"
          aria-expanded={open && !!query}
          aria-controls="journal-results"
          aria-autocomplete="list"
          aria-activedescendant={
            open && results[index] ? `result-${index}` : undefined
          }
          onFocus={() => setOpen(true)}
          onChange={(event) => {
            setQuery(event.target.value);
            setOpen(true);
            setIndex(0);
          }}
          onKeyDown={(event) => {
            if (event.key === "ArrowDown") {
              event.preventDefault();
              setOpen(true);
              setIndex((i) =>
                Math.max(0, Math.min(i + 1, Math.min(results.length, 40) - 1)),
              );
            }
            if (event.key === "ArrowUp") {
              event.preventDefault();
              setIndex((i) => Math.max(0, i - 1));
            }
            if (event.key === "Enter" && results[index]) {
              event.preventDefault();
              choose(results[index]);
            }
          }}
        />
        <kbd>Ctrl K</kbd>
        {open && query && (
          <div className="search-results" id="journal-results" role="listbox">
            <div className="search-caption">
              {results.length} {tr(language, "results")}
            </div>
            {results.slice(0, 40).map((result, i) => (
              <button
                role="option"
                aria-selected={i === index}
                id={`result-${i}`}
                className={i === index ? "highlighted" : ""}
                key={result.key}
                onMouseEnter={() => setIndex(i)}
                onClick={() => choose(result)}
              >
                <Artwork token={result.art} kind={result.category} />
                <span>
                  <strong>{result.name}</strong>
                  <small>
                    {tr(language, result.category)} ·{" "}
                    {tr(language, result.local ? "thisPage" : "journal")}
                  </small>
                </span>
                <Icon name="chevron" size={14} />
              </button>
            ))}
            {!results.length && <p>{tr(language, "noResults")}</p>}
          </div>
        )}
      </div>
      <details className="filter-menu">
        <summary>
          <Icon name="filter" size={17} />
          <span>{tr(language, "filters")}</span>
          {(filters.season || filters.place || filters.state !== "all") && (
            <i />
          )}
        </summary>
        <div className="filter-popover">
          <label>
            {tr(language, "season")}
            <select
              disabled={!availabilityEnabled}
              value={filters.season}
              onChange={(event) =>
                onFilters({ ...filters, season: event.target.value })
              }
            >
              <option value="">{tr(language, "allSeasons")}</option>
              {["spring", "summer", "fall", "winter"].map((season) => (
                <option key={season} value={season}>
                  {tr(language, season)}
                </option>
              ))}
            </select>
          </label>
          <label>
            {tr(language, "place")}
            <select
              disabled={!availabilityEnabled}
              value={filters.place}
              onChange={(event) =>
                onFilters({ ...filters, place: event.target.value })
              }
            >
              <option value="">{tr(language, "allPlaces")}</option>
              {places.map((place) => (
                <option key={place} value={place}>
                  {tr(language, place)}
                </option>
              ))}
            </select>
          </label>
          <label>
            {tr(language, view === "museum" ? "donated" : "found")}
            <select
              disabled={!filtersEnabled}
              value={filters.state}
              onChange={(event) =>
                onFilters({
                  ...filters,
                  state: event.target.value as Filters["state"],
                })
              }
            >
              {["all", "found", "missing"].map((state) => (
                <option value={state} key={state}>
                  {tr(
                    language,
                    view === "museum" && state !== "all"
                      ? state === "found"
                        ? "donated"
                        : "notDonated"
                      : state,
                  )}
                </option>
              ))}
            </select>
          </label>
          <button
            className="text-button"
            onClick={() => onFilters(defaultFilters)}
          >
            {tr(language, "reset")}
          </button>
        </div>
      </details>
      <select
        disabled={!filtersEnabled}
        className="sort-select"
        aria-label={tr(language, "sort")}
        value={filters.sort}
        onChange={(event) =>
          onFilters({ ...filters, sort: event.target.value as Filters["sort"] })
        }
      >
        {[
          ["name", "nameAsc"],
          ["name-desc", "nameDesc"],
          ["found", "foundFirst"],
          ["missing", "missingFirst"],
        ].map(([value, label]) => (
          <option key={value} value={value}>
            {tr(language, label)}
          </option>
        ))}
      </select>
    </header>
  );
}
