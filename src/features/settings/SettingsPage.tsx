import { localeOptions, type LanguagePreference, type SupportedLocale } from "../../i18n";
import { tr } from "../../journal/copy";

export type SpoilerMode = "all" | "free";
export type Language = SupportedLocale;
export type { LanguagePreference } from "../../i18n";

export type TrackerSettings = {
  spoilerMode: SpoilerMode;
  hintsEnabled: boolean;
};

export type LiveReadiness = {
  state: "idle" | "checking" | "complete" | "error";
  catalogApproved?: boolean;
  companionLogFound?: boolean;
};

export type ExistingImport = {
  state: "idle" | "importing" | "complete" | "error";
  discoveredItems?: number;
  imported?: boolean;
};

type SettingsPageProps = TrackerSettings & {
  onChange: (settings: TrackerSettings) => void;
  language?: Language;
  languagePreference?: LanguagePreference;
  onLanguagePreferenceChange?: (preference: LanguagePreference) => void;
  liveReadiness?: LiveReadiness;
  existingImport?: ExistingImport;
  onImportExisting?: () => void;
};

export function SettingsPage({
  spoilerMode,
  hintsEnabled,
  onChange,
  language = "eng",
  languagePreference = "auto",
  onLanguagePreferenceChange,
  liveReadiness = { state: "idle" },
  existingImport = { state: "idle" },
  onImportExisting,
}: SettingsPageProps) {
  const text = (key: string) => tr(language, key);
  const readinessMessage =
    liveReadiness.state === "checking"
      ? text("checkingReadiness")
      : liveReadiness.state === "error"
        ? text("failedReadiness")
        : liveReadiness.state === "complete"
          ? liveReadiness.catalogApproved
            ? liveReadiness.companionLogFound
              ? text("readyReadiness")
              : text("missingLogReadiness")
            : text("blockedReadiness")
          : text("idleReadiness");
  const importMessage =
    existingImport.state === "importing"
      ? text("importing")
      : existingImport.state === "complete" &&
          existingImport.discoveredItems !== undefined
        ? tr(
            language,
            existingImport.imported
              ? "importedDiscoveries"
              : "alreadyImportedDiscoveries",
            { count: existingImport.discoveredItems },
          )
        : existingImport.state === "error"
          ? text("importFailed")
          : null;

  return (
    <section aria-labelledby="settings-heading">
      <p className="eyebrow">{text("settingsEyebrow")}</p>
      <h1 id="settings-heading">{text("settings")}</h1>

      <fieldset>
        <legend>{text("spoilerMode")}</legend>
        <label>
          <input
            checked={spoilerMode === "free"}
            name="spoiler-mode"
            onChange={() => onChange({ spoilerMode: "free", hintsEnabled: false })}
            type="radio"
          />
          {text("spoilerFree")}
        </label>
        <label>
          <input
            checked={spoilerMode === "all"}
            name="spoiler-mode"
            onChange={() => onChange({ spoilerMode: "all", hintsEnabled: false })}
            type="radio"
          />
          {text("showAll")}
        </label>
      </fieldset>

      <label>
        <input
          checked={hintsEnabled}
          disabled={spoilerMode !== "free"}
          onChange={(event) =>
            onChange({ spoilerMode, hintsEnabled: event.currentTarget.checked })
          }
          type="checkbox"
        />
        {text("enableHints")}
      </label>
      <p className="muted-copy">{text("hintsHelp")}</p>

      <section aria-labelledby="live-safety-heading">
        <h2 id="live-safety-heading">{text("liveSafety")}</h2>
        <p aria-live="polite" className="muted-copy">
          {readinessMessage}
        </p>
      </section>

      {onImportExisting && (
        <section aria-labelledby="existing-discoveries-heading">
          <h2 id="existing-discoveries-heading">{text("existingDiscoveries")}</h2>
          <p className="muted-copy">{text("importHelp")}</p>
          <button
            disabled={existingImport.state === "importing"}
            onClick={onImportExisting}
            type="button"
          >
            {text("importExisting")}
          </button>
          {importMessage && (
            <p aria-live="polite" className="muted-copy">
              {importMessage}
            </p>
          )}
        </section>
      )}

      <fieldset>
        <legend>{text("language")}</legend>
        <p className="muted-copy">{text("autoDetectHelp")}</p>
        <label>
          <input
            checked={languagePreference === "auto"}
            name="tracker-language"
            onChange={() => onLanguagePreferenceChange?.("auto")}
            type="radio"
          />
          {text("autoDetect")}
        </label>
        {localeOptions.map((option) => (
          <label key={option.value}>
            <input
              checked={languagePreference === option.value}
              name="tracker-language"
              onChange={() => onLanguagePreferenceChange?.(option.value)}
              type="radio"
            />
            {text(option.label)}
          </label>
        ))}
      </fieldset>
    </section>
  );
}
