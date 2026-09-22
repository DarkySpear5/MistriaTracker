import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { join, localDataDir } from "@tauri-apps/api/path";
import {
  getPreferences,
  getProfileSummary,
  chooseAndImportSave,
  isNativeTrackerRuntime,
  pollLiveTracking,
  probeReadiness,
  resolveGameDirectory,
  savePreferences,
  startLiveTracking,
  chooseAndSaveGameDirectory,
} from "../lib/nativeTracker";
import { ArtworkProvider } from "../journal/Artwork";
import { HintCard } from "../journal/HintCard";
import { Icon } from "../journal/Icon";
import { DetailPanel } from "../journal/DetailPanel";
import { Toolbar } from "../journal/Toolbar";
import {
  Encyclopedia,
  Museum,
  Overview,
  PageHeading,
  Villagers,
} from "../journal/screens";
import type { SearchResult } from "../journal/search";
import { tr } from "../journal/copy";
import {
  languageDictionary,
  localeOptions,
  type LanguagePreference,
} from "../i18n";
import {
  defaultFilters,
  type Filters,
  type Journal,
  type Language,
  type Hint,
  type Source,
  type View,
} from "../journal/types";

type Props = {
  language?: Language;
  nativeRuntime?: () => boolean;
  readinessProbe?: typeof probeReadiness;
  importExisting?: typeof chooseAndImportSave;
  resolveGameDirectory?: typeof resolveGameDirectory;
  chooseGameDirectory?: typeof chooseAndSaveGameDirectory;
  initialJournal?: Journal;
  artLoader?: (keys: string[]) => Promise<Record<string, string | null>>;
};
export function DesktopApp({
  language: initialLanguage = "eng",
  nativeRuntime = isNativeTrackerRuntime,
  readinessProbe = probeReadiness,
  importExisting = chooseAndImportSave,
  resolveGameDirectory: resolveGameDirectoryForApp = resolveGameDirectory,
  chooseGameDirectory = chooseAndSaveGameDirectory,
  initialJournal,
  artLoader,
}: Props) {
  const native = nativeRuntime();
  const [language, setLanguage] = useState(initialLanguage);
  const [languagePreference, setLanguagePreference] =
    useState<LanguagePreference>("auto");
  const [spoilers, setSpoilers] = useState(false);
  const [hints, setHints] = useState(false);
  const [journal, setJournal] = useState<Journal | null>(
    initialJournal ?? null,
  );
  const [view, setView] = useState<View>("overview");
  const [category, setCategory] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [hint, setHint] = useState<Hint | null>(null);
  const [filters, setFilters] = useState<Filters>(defaultFilters);
  const [busy, setBusy] = useState(native && !initialJournal);
  const [status, setStatus] = useState("waiting");
  const [error, setError] = useState("");
  const [saveImportMessage, setSaveImportMessage] = useState("");
  const [profileKey, setProfileKey] = useState("");
  const [gameDirectory, setGameDirectory] = useState<string | null>(null);
  const activeProfileRef = useRef("");
  const noteDrafts = useRef(new Map<string, string>());
  const scrollRef = useRef<HTMLDivElement>(null);
  const sessionStarted = useRef(false);
  const initializeRef = useRef<(selectedGameDirectory?: string) => Promise<void>>(
    async () => {},
  );
  const refresh = async () => {
    const [snapshot, profiles] = await Promise.all([
      invoke<Journal | null>("get_journal_snapshot"),
      getProfileSummary(),
    ]);
    setJournal(snapshot);
    const nextProfile = snapshot?.profile.id ?? profiles.active_profile ?? "";
    activeProfileRef.current = nextProfile;
    setProfileKey(nextProfile);
  };
  useEffect(() => {
    if (!native) return;
    let alive = true;
    let timer: number | undefined;
    let polling = false;
    let initializing = false;
    let modDataDirectory = "";
    let retryTicks = 0;
    const start = async (selectedGameDirectory?: string) => {
      if (initializing || !alive) return;
      initializing = true;
      setBusy(true);
      setError("");
      try {
        const preferences = await getPreferences();
        if (!alive) return;
        setLanguage(preferences.effective_language);
        setLanguagePreference(preferences.language_preference);
        setSpoilers(preferences.spoiler_mode === "all");
        setHints(preferences.hints_enabled);
        const resolvedGameDirectory =
          selectedGameDirectory ?? (await resolveGameDirectoryForApp());
        if (!resolvedGameDirectory) {
          setStatus("waiting");
          setError("game-directory");
          return;
        }
        setGameDirectory(resolvedGameDirectory);
        const modData = await join(
          await localDataDir(),
          "FieldsOfMistria",
          "mod_data",
        );
        const report = await readinessProbe(resolvedGameDirectory, modData);
        modDataDirectory = modData;
        if (!alive) return;
        if (report.catalog_approved) {
          if (report.companion_log_found && !sessionStarted.current) {
            await startLiveTracking(resolvedGameDirectory, modData);
            sessionStarted.current = true;
          } else if (!sessionStarted.current)
            await invoke("prepare_journal", { gameDirectory: resolvedGameDirectory });
          setStatus(report.companion_log_found ? "tracking" : "waiting");
          await refresh();
        } else {
          setStatus("offline");
          setError("compatibility");
        }
      } catch {
        if (alive) {
          setStatus("offline");
          setError("startup");
        }
      } finally {
        initializing = false;
        if (alive) setBusy(false);
      }
    };
    initializeRef.current = start;
    void start();
    timer = window.setInterval(() => {
      if (!alive || initializing || polling) return;
      if (!sessionStarted.current) {
        if (!modDataDirectory || ++retryTicks < 10) return;
        retryTicks = 0;
        polling = true;
        void invoke<boolean>("companion_log_available", { modDataDirectory })
          .then(async (available) => {
            if (available && alive) await start();
          })
          .catch(() => {})
          .finally(() => {
            polling = false;
          });
        return;
      }
      polling = true;
      void pollLiveTracking()
        .then(async (count) => {
          if (!alive) return;
          setStatus("tracking");
          if (count) await refresh();
          else {
            const profiles = await getProfileSummary();
            if (
              alive &&
              (profiles.active_profile ?? "") !== activeProfileRef.current
            )
              await refresh();
          }
        })
        .catch(() => alive && setStatus("offline"))
        .finally(() => {
          polling = false;
        });
    }, 1500);
    return () => {
      alive = false;
      if (timer) window.clearInterval(timer);
    };
  }, []);
  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.key === "Escape") setHint(null);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
  const dirtyNote = useRef(false);
  const onDirtyChange = useCallback((dirty: boolean) => {
    dirtyNote.current = dirty;
  }, []);
  const canLeave = () => {
    if (
      dirtyNote.current &&
      !window.confirm(
        tr(language, "leaveWithoutSaving"),
      )
    )
      return false;
    dirtyNote.current = false;
    return true;
  };
  const openEntry = (key: string) => {
    if (key !== selected && !canLeave()) return;
    setSelected(key);
  };
  const navigate = (next: View, nextCategory?: string) => {
    if (!canLeave()) return;
    setView(next);
    setCategory(nextCategory ?? null);
    setFilters(defaultFilters);
    setSelected(null);
    scrollRef.current?.scrollTo?.({ top: 0 });
  };
  const persist = async (
    nextLanguagePreference: LanguagePreference,
    nextSpoilers: boolean,
    nextHints: boolean,
  ) => {
    if (!canLeave()) return;
    setSelected(null);
    setHint(null);
    if (!native) {
      if (nextLanguagePreference !== "auto") setLanguage(nextLanguagePreference);
      setLanguagePreference(nextLanguagePreference);
      setSpoilers(nextSpoilers);
      setHints(nextHints);
      return;
    }
    setBusy(true);
    try {
      await savePreferences({
        language_preference: nextLanguagePreference,
        spoiler_mode: nextSpoilers ? "all" : "free",
        hints_enabled: nextHints,
      });
      const updatedPreferences = await getPreferences();
      setLanguage(updatedPreferences.effective_language);
      setLanguagePreference(updatedPreferences.language_preference);
      setSpoilers(nextSpoilers);
      setHints(nextHints);
      await refresh();
      setError("");
    } catch {
      setError("preferences");
    } finally {
      setBusy(false);
    }
  };
  const chooseGameDirectoryAndRestart = async () => {
    if (!native) return;
    setBusy(true);
    setError("");
    try {
      const selectedGameDirectory = await chooseGameDirectory();
      if (selectedGameDirectory) await initializeRef.current(selectedGameDirectory);
      else setBusy(false);
    } catch {
      setStatus("waiting");
      setError("game-directory");
      setBusy(false);
    }
  };
  const entry = journal?.entries.find(
    (item) => item.key === selected && item.revealed,
  );
  const chooseResult = (result: SearchResult) => {
    if (!canLeave()) return;
    if (result.target !== view)
      navigate(
        result.target,
        result.type === "entry" ? result.category : undefined,
      );
    if (result.type === "entry") {
      if (result.target === "encyclopedia") setCategory(result.category);
      setSelected(result.key);
    } else {
      setSelected(null);
      setTimeout(
        () =>
          document
            .getElementById(`villager-${result.key}`)
            ?.scrollIntoView({ block: "center" }),
        0,
      );
    }
  };
  const source = (value: Source) => {
    if (!canLeave()) return;
    navigate(value.view, value.view === "encyclopedia" ? value.key : undefined);
    if (value.entry) setSelected(value.entry);
    if (value.view === "museum" || value.view === "villagers")
      setTimeout(
        () =>
          document
            .getElementById(
              `${value.view === "museum" ? "set" : "villager"}-${value.key}`,
            )
            ?.scrollIntoView({ block: "center" }),
        0,
      );
  };
  const showHint = (value: Hint) => {
    if (hints && !spoilers) setHint(value);
  };
  const screen = journal
    ? {
        journal,
        language,
        filters,
        onEntry: openEntry,
        onHint: showHint,
        onNavigate: navigate,
      }
    : null;
  const languageCopy = languageDictionary(language);
  const scope = `${profileKey}:${language}:${spoilers}`;
  return (
    <ArtworkProvider loader={artLoader} scope={scope}>
      <div className="desktop-shell">
        <a className="skip-link" href="#content">
          {tr(language, "skipToContent")}
        </a>
        <aside className="sidebar">
          <div className="brand">
            <div className="brand-symbol">
              <Icon name="moon" size={25} />
            </div>
            <div>
              Mistria<span>{tr(language, "brandTagline")}</span>
            </div>
          </div>
          <div className="sidebar-divider" />
          <nav aria-label={tr(language, "trackerNavigation")}>
            {(
              ["overview", "museum", "villagers", "encyclopedia"] as View[]
            ).map((tab) => (
              <button
                key={tab}
                className={`nav-item ${view === tab ? "active" : ""}`}
                aria-current={view === tab ? "page" : undefined}
                onClick={() => navigate(tab)}
              >
                <Icon name={tab} />
                {tr(language, tab)}
                {tab === "museum" && journal && (
                  <small>
                    {journal.sets.reduce((sum, set) => sum + set.completed, 0)}
                  </small>
                )}
              </button>
            ))}
          </nav>
          <div className="sidebar-bottom">
            <div className="spoiler-indicator">
              <Icon name={spoilers ? "eye" : "lock"} size={14} />
              {tr(language, spoilers ? "showAll" : "spoilerFree")}
            </div>
            <button
              className={`nav-item ${view === "settings" ? "active" : ""}`}
              onClick={() => navigate("settings")}
            >
              <Icon name="settings" />
              {tr(language, "settings")}
            </button>
            <div className="profile-strip">
              <span className="profile-avatar">
                <Icon name="leaf" size={20} />
              </span>
              <div>
                <strong>{journal?.profile.name || "Mistria Tracker"}</strong>
                <small>{journal?.profile.farm || tr(language, "local")}</small>
              </div>
              <span
                className={`connection-dot ${status === "tracking" ? "connected" : ""}`}
                title={tr(language, status)}
              />
            </div>
          </div>
        </aside>
        <div className="main-shell">
          <Toolbar
            key={scope}
            journal={journal}
            language={language}
            view={view}
            category={category}
            filters={filters}
            onFilters={setFilters}
            onResult={chooseResult}
          />
          <div className="workspace">
            <main
              id="content"
              className="content-scroll"
              ref={scrollRef}
              aria-busy={busy}
            >
              {!busy && error && (
                <div className="notice" role="status">
                  <Icon name="info" size={16} />
                  <span>
                    {tr(language, error === "game-directory" ? "gameFolderNotFound" : "refreshPreserved")}
                  </span>
                  <button onClick={() => void initializeRef.current()}>
                    {tr(language, "retry")}
                  </button>
                </div>
              )}
              {view === "settings" ? (
                <>
                  <PageHeading
                    title={tr(language, "settings")}
                    subtitle={tr(language, "spoilerHelp")}
                  />
                  <div className="settings-card">
                    <h2>{tr(language, "appearance")}</h2>
                    <label className="setting-row">
                      <span>
                        <strong>{tr(language, "spoilerFree")}</strong>
                        <small>{tr(language, "spoilerHelp")}</small>
                      </span>
                      <input
                        type="radio"
                        name="spoilers"
                        checked={!spoilers}
                        onChange={() => void persist(languagePreference, false, hints)}
                      />
                    </label>
                    <label className="setting-row">
                      <span>
                        <strong>{tr(language, "showAll")}</strong>
                        <small>{tr(language, "showAllHelp")}</small>
                      </span>
                      <input
                        type="radio"
                        name="spoilers"
                        checked={spoilers}
                        onChange={() => void persist(languagePreference, true, false)}
                      />
                    </label>
                    <label className="setting-row">
                      <span>
                        <strong>{tr(language, "hints")}</strong>
                        <small>{tr(language, "hintsHelp")}</small>
                      </span>
                      <input
                        type="checkbox"
                        role="switch"
                        checked={hints}
                        disabled={spoilers}
                        onChange={(event) =>
                          void persist(languagePreference, spoilers, event.target.checked)
                        }
                      />
                    </label>
                  </div>
                  <div className="settings-card">
                    <h2>{tr(language, "language")}</h2>
                    <p>{languageCopy.autoDetectHelp}</p>
                    <div className="language-buttons">
                      <button
                        aria-pressed={languagePreference === "auto"}
                        onClick={() => void persist("auto", spoilers, hints)}
                      >
                        {languageCopy.autoDetect}
                      </button>
                      {localeOptions.map((option) => (
                        <button
                          key={option.value}
                          aria-pressed={languagePreference === option.value}
                          onClick={() => void persist(option.value, spoilers, hints)}
                        >
                          {languageCopy[option.label]}
                        </button>
                      ))}
                    </div>
                  </div>
                  <div className="settings-card">
                    <h2>{tr(language, "existingDiscoveries")}</h2>
                    <p>{tr(language, "importHelp")}</p>
                    <button
                      className="secondary-button"
                      disabled={busy || !native}
                      onClick={() => {
                        setBusy(true);
                        setSaveImportMessage("");
                        void importExisting()
                          .then(async (result) => {
                            if (!result) return;
                            if (result.status === "unsupported_version") {
                              setSaveImportMessage(
                                tr(language, "unsupportedSaveVersion", { version: result.game_version ?? "" }),
                              );
                              return;
                            }
                            setSaveImportMessage(tr(language, result.imported ? "importedDiscoveries" : "alreadyImportedDiscoveries", { count: String(result.discovered_items) }));
                            await refresh();
                          })
                          .catch(() => setError("import"))
                          .finally(() => setBusy(false));
                      }}
                    >
                      {tr(language, "import")}
                    </button>
                    {saveImportMessage && (
                      <p className="muted-copy" aria-live="polite">
                        {saveImportMessage}
                      </p>
                    )}
                  </div>
                  {!gameDirectory && native && (
                    <div className="settings-card">
                      <h2>{tr(language, "gameFolderTitle")}</h2>
                      <p>{tr(language, "gameFolderHelp")}</p>
                      <button
                        className="secondary-button"
                        disabled={busy}
                        onClick={() => void chooseGameDirectoryAndRestart()}
                      >
                        {tr(language, "chooseGameFolder")}
                      </button>
                    </div>
                  )}
                </>
              ) : screen ? (
                view === "overview" ? (
                  <Overview {...screen} />
                ) : view === "museum" ? (
                  <Museum {...screen} />
                ) : view === "villagers" ? (
                  <Villagers {...screen} />
                ) : (
                  <Encyclopedia
                    key={category ?? "categories"}
                    {...screen}
                    category={category}
                  />
                )
              ) : !busy ? (
                <div className="empty-journal">
                  <Icon name="encyclopedia" size={60} />
                  <h1>{tr(language, "noData")}</h1>
                  <p>{tr(language, "noDataHelp")}</p>
                  <button
                    className="primary-button"
                    onClick={() => void initializeRef.current()}
                  >
                    {tr(language, "retry")}
                  </button>
                </div>
              ) : null}
              <footer className="content-footer">
                <Icon name="leaf" size={12} />
                {tr(language, "local")}
              </footer>
            </main>
            {entry && (
              <DetailPanel
                key={`${profileKey}:${entry.key}`}
                profileId={profileKey}
                draft={noteDrafts.current.get(`${profileKey}:${entry.key}`)}
                onDraftChange={(value) => {
                  const key = `${profileKey}:${entry.key}`;
                  if (value === undefined) noteDrafts.current.delete(key);
                  else noteDrafts.current.set(key, value);
                }}
                entry={entry}
                language={language}
                onClose={() => setSelected(null)}
                onSource={source}
                onEntry={openEntry}
                onDirtyChange={onDirtyChange}
                native={native}
              />
            )}
          </div>
        </div>
        {busy && (
          <div className="loading-screen" role="status">
            <div className="loading-card">
              <div className="loading-moon">
                <Icon name="moon" size={40} />
              </div>
              <h2>{tr(language, "preparing")}</h2>
              <p>{tr(language, "preparingHelp")}</p>
              <div className="loading-track">
                <span />
              </div>
            </div>
          </div>
        )}
        {hint && (
          <div className="modal-backdrop" onClick={() => setHint(null)}>
            <div
              className="hint-card"
              role="dialog"
              aria-modal="true"
              aria-labelledby="hint-title"
              onClick={(event) => event.stopPropagation()}
            >
              <HintCard hint={hint} language={language} onClose={() => setHint(null)} />
            </div>
          </div>
        )}
      </div>
    </ArtworkProvider>
  );
}
