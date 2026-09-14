export type SpoilerMode = 'all' | 'free';
export type Language = 'eng' | 'fra';

const copy = {
  eng: {
    eyebrow: 'Tracker preferences', title: 'Settings', spoilerMode: 'Spoiler mode', spoilerFree: 'Spoiler-free', showEverything: 'Show everything', hints: 'Enable gentle hints', hintHelp: 'Hints are reserved for a later update. They will never reveal an item name or exact location.', language: 'Language', english: 'English', french: 'Français', liveSafety: 'Live tracking safety', checkReadiness: 'Run read-only readiness check', idleReadiness: 'The tracker is paused. Checking readiness never reads or writes a save file.', checkingReadiness: 'Checking the installed game files and isolated companion log…', readyReadiness: 'Catalog approved. The companion log is ready for passive tracking.', missingLogReadiness: 'Catalog approved, but the companion has not created its isolated log yet.', blockedReadiness: 'This game build is not approved for live tracking. The tracker remains paused.', failedReadiness: 'The readiness check could not complete. The tracker remains paused.', existingDiscoveries: 'Existing discoveries', importExisting: 'Import existing discoveries', importHelp: 'Reads only your newest copy in Mistria Save Backups. Your live game save is never opened or changed.', importing: 'Reading your backup copy…', imported: (count: number) => `${count} existing discoveries are now in your local tracker.`, alreadyImported: (count: number) => `${count} existing discoveries were already imported from this backup.`, importFailed: 'The backup import could not complete. Your game save was not touched.',
  },
  fra: {
    eyebrow: 'Préférences du suivi', title: 'Réglages', spoilerMode: 'Mode spoiler', spoilerFree: 'Sans spoiler', showEverything: 'Tout afficher', hints: 'Activer les indices légers', hintHelp: 'Les indices sont réservés à une future mise à jour. Ils ne révéleront jamais le nom ni l’emplacement exact d’un objet.', language: 'Langue', english: 'English', french: 'Français', liveSafety: 'Sécurité du suivi en direct', checkReadiness: 'Lancer le contrôle de disponibilité en lecture seule', idleReadiness: 'Le suivi est en pause. Ce contrôle ne lit ni n’écrit aucun fichier de sauvegarde.', checkingReadiness: 'Vérification des fichiers du jeu et du journal isolé du compagnon…', readyReadiness: 'Catalogue approuvé. Le journal du compagnon est prêt pour le suivi passif.', missingLogReadiness: 'Catalogue approuvé, mais le compagnon n’a pas encore créé son journal isolé.', blockedReadiness: 'Cette version du jeu n’est pas approuvée pour le suivi en direct. Le suivi reste en pause.', failedReadiness: 'Le contrôle de disponibilité n’a pas pu aboutir. Le suivi reste en pause.', existingDiscoveries: 'Découvertes existantes', importExisting: 'Importer mes découvertes existantes', importHelp: 'Lit seulement votre copie la plus récente dans Mistria Save Backups. Votre sauvegarde du jeu n’est jamais ouverte ni modifiée.', importing: 'Lecture de votre copie de sauvegarde…', imported: (count: number) => `${count} découvertes existantes sont maintenant dans votre suivi local.`, alreadyImported: (count: number) => `${count} découvertes existantes avaient déjà été importées depuis cette copie.`, importFailed: 'L’import de la copie n’a pas pu aboutir. Votre sauvegarde du jeu n’a pas été touchée.',
  },
} as const;

export type TrackerSettings = {
  spoilerMode: SpoilerMode;
  hintsEnabled: boolean;
};

export type LiveReadiness = {
  state: 'idle' | 'checking' | 'complete' | 'error';
  catalogApproved?: boolean;
  companionLogFound?: boolean;
};

export type ExistingImport = {
  state: 'idle' | 'importing' | 'complete' | 'error';
  discoveredItems?: number;
  imported?: boolean;
};

type SettingsPageProps = TrackerSettings & {
  onChange: (settings: TrackerSettings) => void;
  language?: Language;
  onLanguageChange?: (language: Language) => void;
  liveReadiness?: LiveReadiness;
  existingImport?: ExistingImport;
  onImportExisting?: () => void;
};

export function SettingsPage({ spoilerMode, hintsEnabled, onChange, language, onLanguageChange, liveReadiness = { state: 'idle' }, existingImport = { state: 'idle' }, onImportExisting }: SettingsPageProps) {
  const text = copy[language ?? 'eng'];
  const readinessMessage = liveReadiness.state === 'checking'
    ? text.checkingReadiness
    : liveReadiness.state === 'error'
      ? text.failedReadiness
      : liveReadiness.state === 'complete'
        ? liveReadiness.catalogApproved
          ? liveReadiness.companionLogFound ? text.readyReadiness : text.missingLogReadiness
          : text.blockedReadiness
        : text.idleReadiness;
  const importMessage = existingImport.state === 'importing'
    ? text.importing
    : existingImport.state === 'complete' && existingImport.discoveredItems !== undefined
      ? existingImport.imported ? text.imported(existingImport.discoveredItems) : text.alreadyImported(existingImport.discoveredItems)
      : existingImport.state === 'error' ? text.importFailed : null;
  return (
    <section aria-labelledby="settings-heading">
      <p className="eyebrow">{text.eyebrow}</p>
      <h1 id="settings-heading">{text.title}</h1>

      <fieldset>
        <legend>{text.spoilerMode}</legend>
        <label>
          <input
            checked={spoilerMode === 'free'}
            name="spoiler-mode"
            onChange={() => onChange({ spoilerMode: 'free', hintsEnabled: false })}
            type="radio"
          />
          {text.spoilerFree}
        </label>
        <label>
          <input
            checked={spoilerMode === 'all'}
            name="spoiler-mode"
            onChange={() => onChange({ spoilerMode: 'all', hintsEnabled: false })}
            type="radio"
          />
          {text.showEverything}
        </label>
      </fieldset>

      <label>
        <input
          checked={hintsEnabled}
          disabled={spoilerMode !== 'free'}
          onChange={(event) =>
            onChange({ spoilerMode, hintsEnabled: event.currentTarget.checked })
          }
          type="checkbox"
        />
        {text.hints}
      </label>
      <p className="muted-copy">
        {text.hintHelp}
      </p>

      <section aria-labelledby="live-safety-heading">
        <h2 id="live-safety-heading">{text.liveSafety}</h2>
        <p aria-live="polite" className="muted-copy">{readinessMessage}</p>
      </section>

      {onImportExisting && <section aria-labelledby="existing-discoveries-heading">
        <h2 id="existing-discoveries-heading">{text.existingDiscoveries}</h2>
        <p className="muted-copy">{text.importHelp}</p>
        <button disabled={existingImport.state === 'importing'} onClick={onImportExisting} type="button">
          {text.importExisting}
        </button>
        {importMessage && <p aria-live="polite" className="muted-copy">{importMessage}</p>}
      </section>}

      <fieldset>
        <legend>{text.language}</legend>
        <label>
          <input
            checked={language === 'eng'}
            name="tracker-language"
            onChange={() => onLanguageChange?.('eng')}
            type="radio"
          />
          {text.english}
        </label>
        <label>
          <input
            checked={language === 'fra'}
            name="tracker-language"
            onChange={() => onLanguageChange?.('fra')}
            type="radio"
          />
          {text.french}
        </label>
      </fieldset>
    </section>
  );
}
