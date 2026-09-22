use crate::{
    catalog::{probe, AssetsZip, Catalog, CatalogCacheDir, CatalogExtractor, CatalogProbeReport},
    compatibility::{CompatibilityDecision, CompatibilityMatrix, SaveParserDecision, VersionSet},
    domain::{CompanionEvent, EntityId, EventEnvelope, ItemId, Language, ProfileId, SpoilerMode},
    localization::{self, LanguagePreference},
    persistence::{AcceptResult, Database, ImportResult, RepoError, Repository},
    safety::paths::GameAssetsPath,
    spoilers::{AppSnapshot, SpoilerPolicy},
    steam_discovery::{
        discover_game_directory, discover_windows_game_directory, validate_game_directory,
        DiscoveryResult, DiscoverySources,
    },
    tracking::{
        log_lines::event_json_from_log_line, LogTailError, ProfileProgress, ReadOnlyLogTail,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Debug, thiserror::Error)]
pub enum TrackerStateError {
    #[error("tracker state is unavailable")]
    Unavailable,
    #[error("no local tracker profile is selected")]
    NoActiveProfile,
    #[error(transparent)]
    Repository(#[from] RepoError),
    #[error("invalid profile identifier")]
    InvalidProfile,
    #[error("invalid item identifier")]
    InvalidItem,
    #[error("companion event is invalid: {0}")]
    Event(#[from] crate::domain::EventError),
    #[error("live tracking is paused until this game version is explicitly approved")]
    LiveTrackingPaused,
    #[error("catalog loading is paused until this game version is explicitly approved")]
    CatalogPaused,
    #[error(transparent)]
    Catalog(#[from] crate::catalog::CatalogError),
    #[error(transparent)]
    GamePath(#[from] crate::safety::paths::GameSavePathError),
    #[error(transparent)]
    LogTail(#[from] LogTailError),
    #[error("the selected Fields of Mistria folder must contain a readable assets.zip file")]
    InvalidGameDirectory,
}

pub struct TrackerState {
    repository: Mutex<Repository>,
    catalog: Mutex<Option<Catalog>>,
    journal: Mutex<Option<(crate::journal::catalog::JournalCatalog, std::path::PathBuf)>>,
}

const PREFERENCES_KEY: &str = "tracker_preferences";
const CATALOG_PARSER_VERSION: u16 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TrackerPreferences {
    pub language_preference: LanguagePreference,
    pub spoiler_mode: SpoilerMode,
    pub hints_enabled: bool,
    #[serde(default)]
    pub game_directory: Option<PathBuf>,
}

#[derive(Deserialize)]
struct StoredTrackerPreferences {
    #[serde(default)]
    language_preference: Option<LanguagePreference>,
    #[serde(default)]
    language: Option<Language>,
    #[serde(default)]
    spoiler_mode: Option<SpoilerMode>,
    #[serde(default)]
    hints_enabled: Option<bool>,
    #[serde(default)]
    game_directory: Option<PathBuf>,
}

impl<'de> Deserialize<'de> for TrackerPreferences {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let stored = StoredTrackerPreferences::deserialize(deserializer)?;
        // Versions before unified language support stored a direct UI language.
        // Treat it as Auto so the tracker can match the selected Steam language.
        let _legacy_language = stored.language;
        Ok(Self {
            language_preference: stored.language_preference.unwrap_or_default(),
            spoiler_mode: stored.spoiler_mode.unwrap_or(SpoilerMode::Free),
            hints_enabled: stored.hints_enabled.unwrap_or(false),
            game_directory: stored.game_directory,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReadinessReport {
    pub catalog: CatalogProbeReport,
    pub catalog_approved: bool,
    pub companion_log_found: bool,
}

impl Default for TrackerPreferences {
    fn default() -> Self {
        Self {
            language_preference: LanguagePreference::Auto,
            spoiler_mode: SpoilerMode::Free,
            hints_enabled: false,
            game_directory: None,
        }
    }
}

impl TrackerPreferences {
    fn normalized(mut self) -> Self {
        if self.spoiler_mode == SpoilerMode::All {
            self.hints_enabled = false;
        }
        self
    }
}

impl TrackerState {
    /// Opens only the tracker's SQLite database beneath its own data directory.
    /// This type never receives a game-save path.
    pub fn open(tracker_data_dir: &Path) -> Result<Self, TrackerStateError> {
        let database = Database::open(&tracker_data_dir.join("tracker.sqlite"))?;
        Ok(Self::from_repository(database.into_repository()))
    }

    pub fn from_repository(repository: Repository) -> Self {
        Self {
            repository: Mutex::new(repository),
            catalog: Mutex::new(None),
            journal: Mutex::new(None),
        }
    }

    pub fn activate_profile(&self, raw_profile_id: &str) -> Result<(), TrackerStateError> {
        let profile_id =
            ProfileId::new(raw_profile_id).map_err(|_| TrackerStateError::InvalidProfile)?;
        self.repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .activate_profile(&profile_id)?;
        Ok(())
    }

    pub fn active_profile(&self) -> Result<Option<ProfileId>, TrackerStateError> {
        Ok(self
            .repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .active_profile()?)
    }

    pub fn deactivate_profile(&self) -> Result<(), TrackerStateError> {
        self.repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .deactivate_profile()?;
        Ok(())
    }

    pub fn profiles(&self) -> Result<Vec<ProfileId>, TrackerStateError> {
        Ok(self
            .repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .profiles()?)
    }

    pub fn preferences(&self) -> Result<TrackerPreferences, TrackerStateError> {
        let raw = self
            .repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .setting(PREFERENCES_KEY)?;
        let mut preferences = raw
            .and_then(|value| serde_json::from_str::<TrackerPreferences>(&value).ok())
            .unwrap_or_default()
            .normalized();
        preferences.game_directory = preferences
            .game_directory
            .as_deref()
            .and_then(validate_game_directory);
        Ok(preferences)
    }

    pub fn save_preferences(
        &self,
        mut preferences: TrackerPreferences,
    ) -> Result<(), TrackerStateError> {
        let current = self.preferences()?;
        preferences.game_directory = match preferences.game_directory {
            Some(candidate) => Some(
                validate_game_directory(&candidate)
                    .ok_or(TrackerStateError::InvalidGameDirectory)?,
            ),
            None => current.game_directory,
        };
        let serialized =
            serde_json::to_string(&preferences.normalized()).map_err(RepoError::from)?;
        self.repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .save_setting(PREFERENCES_KEY, &serialized)?;
        Ok(())
    }

    /// Resolves the one language used for both Tracker text and catalog names.
    /// A manual selection wins; Auto reads only the game app manifest.
    pub fn effective_language(&self) -> Result<Language, TrackerStateError> {
        let preference = self.preferences()?.language_preference;
        let game_directory = self.resolved_game_directory()?;
        Ok(localization::effective_language(
            preference,
            game_directory.as_deref(),
        ))
    }

    /// Stores a game location only after opening its direct `assets.zip`.
    pub fn save_game_directory(&self, candidate: &Path) -> Result<PathBuf, TrackerStateError> {
        let game_directory =
            validate_game_directory(candidate).ok_or(TrackerStateError::InvalidGameDirectory)?;
        let mut preferences = self.preferences()?;
        preferences.game_directory = Some(game_directory.clone());
        self.save_preferences(preferences)?;
        Ok(game_directory)
    }

    /// Resolves a previously validated choice first, then safe Steam discovery.
    pub fn resolved_game_directory(&self) -> Result<Option<PathBuf>, TrackerStateError> {
        let saved_game = self.preferences()?.game_directory;
        Ok(match discover_windows_game_directory(saved_game) {
            DiscoveryResult::Found(path) => Some(path),
            DiscoveryResult::NotFound => None,
        })
    }

    /// Testable variant of `resolved_game_directory`; production callers use the
    /// Windows-system resolver above.
    pub fn resolved_game_directory_from_sources(
        &self,
        mut sources: DiscoverySources,
    ) -> Result<Option<PathBuf>, TrackerStateError> {
        sources.saved_game = self.preferences()?.game_directory;
        Ok(match discover_game_directory(&sources) {
            DiscoveryResult::Found(path) => Some(path),
            DiscoveryResult::NotFound => None,
        })
    }

    pub fn ingest_companion_event(
        &self,
        event_json: &str,
        compatibility: &CompatibilityMatrix,
    ) -> Result<AcceptResult, TrackerStateError> {
        let event = EventEnvelope::from_json(event_json)?;
        let versions = VersionSet::new(
            event.game_version.clone(),
            event.companion_version.clone(),
            event.schema_version,
        );
        if compatibility.decision(&versions) != CompatibilityDecision::FullySupported {
            return Err(TrackerStateError::LiveTrackingPaused);
        }
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?;
        let activated_save = if matches!(event.event, CompanionEvent::ProfileActivated) {
            Some(event.save_file.clone().unwrap_or_default())
        } else {
            None
        };
        let accepted = repository.accept_event(&event)?;
        if matches!(event.event, CompanionEvent::ProfileDeactivated) {
            repository.deactivate_profile()?;
        } else if matches!(event.event, CompanionEvent::ProfileActivated)
            || repository.active_profile()?.is_none()
        {
            repository.activate_profile(&event.profile_id)?;
        }
        if let Some(save_file) = activated_save {
            repository.save_setting("active_save_file", &save_file)?;
        }
        Ok(accepted)
    }

    pub fn active_save_file(&self) -> Result<Option<String>, TrackerStateError> {
        Ok(self
            .repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .setting("active_save_file")?
            .filter(|value| !value.is_empty()))
    }

    pub fn ingest_companion_log_line(
        &self,
        line: &str,
        compatibility: &CompatibilityMatrix,
    ) -> Result<Option<AcceptResult>, TrackerStateError> {
        let Some(event_json) = event_json_from_log_line(line) else {
            return Ok(None);
        };
        self.ingest_companion_event(event_json, compatibility)
            .map(Some)
    }

    pub fn import_existing_items(
        &self,
        source_hash: [u8; 32],
        profile_id: &ProfileId,
        game_version: &str,
        items: &[ItemId],
    ) -> Result<ImportResult, TrackerStateError> {
        self.repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .import_backup_items(source_hash, profile_id, game_version, items)
            .map_err(TrackerStateError::from)
    }

    pub fn replace_from_save(
        &self,
        source_hash: [u8; 32],
        profile_id: &ProfileId,
        game_version: &str,
        items: &[ItemId],
        journal: &crate::journal::evidence::JournalEvidence,
    ) -> Result<(), TrackerStateError> {
        let journal = serde_json::to_string(journal).map_err(RepoError::from)?;
        self.repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .replace_save_state(source_hash, profile_id, game_version, items, &journal)
            .map_err(TrackerStateError::from)
    }

    /// Polls newly appended complete log lines; each event uses the normal fail-closed gate.
    pub fn poll_companion_log(
        &self,
        tail: &mut ReadOnlyLogTail,
        compatibility: &CompatibilityMatrix,
    ) -> Result<Vec<AcceptResult>, TrackerStateError> {
        tail.poll()?
            .into_iter()
            .filter_map(|line| {
                self.ingest_companion_log_line(&line, compatibility)
                    .transpose()
            })
            .collect()
    }

    /// Installs an already parsed catalog only after its declared game version is approved.
    /// The catalog remains in memory and is never written into a game directory.
    pub fn install_catalog(
        &self,
        catalog: Catalog,
        compatibility: &CompatibilityMatrix,
    ) -> Result<(), TrackerStateError> {
        if compatibility
            .catalog_parser_decision(&catalog.version.source_game_version, CATALOG_PARSER_VERSION)
            != SaveParserDecision::Verified
        {
            return Err(TrackerStateError::CatalogPaused);
        }
        *self
            .catalog
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)? = Some(catalog);
        Ok(())
    }

    /// Reads only an asset archive, then delegates to the compatibility gate before installation.
    pub fn load_catalog(
        &self,
        source: &AssetsZip,
        cache: &CatalogCacheDir,
        compatibility: &CompatibilityMatrix,
    ) -> Result<(), TrackerStateError> {
        let catalog = CatalogExtractor::extract(source, cache)?;
        self.install_catalog(catalog, compatibility)?;
        let journal = crate::journal::catalog::JournalCatalog::extract(source)?;
        let art = crate::journal::art::prepare(source, cache, &journal)?;
        *self
            .journal
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)? = Some((journal, art));
        Ok(())
    }

    pub fn load_catalog_from_game_assets(
        &self,
        assets: &GameAssetsPath,
        cache: &CatalogCacheDir,
        compatibility: &CompatibilityMatrix,
    ) -> Result<(), TrackerStateError> {
        let source = AssetsZip::new(assets.as_path())?;
        self.load_catalog(&source, cache, compatibility)
    }

    /// Inspects only game assets and companion-log presence. It never starts tracking.
    pub fn readiness_report(
        &self,
        game_directory: &Path,
        mod_data_directory: &Path,
        cache: &CatalogCacheDir,
        compatibility: &CompatibilityMatrix,
    ) -> Result<ReadinessReport, TrackerStateError> {
        let assets = GameAssetsPath::new(game_directory, &game_directory.join("assets.zip"))?;
        let source = AssetsZip::new(assets.as_path())?;
        let catalog = probe(&source, cache)?;
        let catalog_approved = compatibility
            .catalog_parser_decision(&catalog.game_version, CATALOG_PARSER_VERSION)
            == SaveParserDecision::Verified;
        let companion_log_found = mod_data_directory
            .join("mistria_tracker_companion/logs/mistria_tracker_companion.log")
            .is_file();
        Ok(ReadinessReport {
            catalog,
            catalog_approved,
            companion_log_found,
        })
    }

    /// Returns a spoiler-filtered view built from tracker-owned event data.
    /// A catalog is absent until its parser has been compatibility-approved.
    pub fn active_profile_snapshot(&self) -> Result<Option<AppSnapshot>, TrackerStateError> {
        let Some(catalog) = self
            .catalog
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .clone()
        else {
            return Ok(None);
        };

        let events = {
            let repository = self
                .repository
                .lock()
                .map_err(|_| TrackerStateError::Unavailable)?;
            let Some(profile_id) = repository.active_profile()? else {
                return Ok(None);
            };
            repository.events(&profile_id)?
        };
        let preferences = self.preferences()?;
        let language = self.effective_language()?;
        let progress = ProfileProgress::from_events(events);
        Ok(Some(SpoilerPolicy.snapshot(
            &progress,
            &catalog,
            preferences.spoiler_mode,
            language,
        )))
    }

    /// Resolves an icon sprite only after the item has passed the same
    /// spoiler-policy boundary as the desktop snapshot.
    pub fn visible_item_icon_sprite(
        &self,
        raw_item_id: &str,
    ) -> Result<Option<String>, TrackerStateError> {
        let item_id = ItemId::new(raw_item_id).map_err(|_| TrackerStateError::InvalidItem)?;
        let Some(snapshot) = self.active_profile_snapshot()? else {
            return Ok(None);
        };
        if !snapshot.items.iter().any(|item| item.id == item_id) {
            return Ok(None);
        }
        let configured_sprite = self
            .catalog
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .as_ref()
            .and_then(|catalog| catalog.item(&item_id))
            .and_then(|item| item.icon_sprite.as_deref())
            .filter(|sprite| valid_item_sprite(sprite))
            .map(str::to_owned);
        Ok(Some(configured_sprite.unwrap_or_else(|| {
            format!("spr_ui_item_{}", item_id.as_str())
        })))
    }

    pub fn store_journal_evidence(
        &self,
        profile: &ProfileId,
        evidence: &crate::journal::evidence::JournalEvidence,
    ) -> Result<(), TrackerStateError> {
        let serialized = serde_json::to_string(evidence).map_err(RepoError::from)?;
        self.repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?
            .save_setting(
                &format!("journal_evidence_v1:{}", profile.as_str()),
                &serialized,
            )?;
        Ok(())
    }

    pub fn journal_snapshot(&self) -> Result<Option<serde_json::Value>, TrackerStateError> {
        self.journal_snapshot_for(None)
    }

    fn journal_snapshot_for(
        &self,
        requested: Option<&ProfileId>,
    ) -> Result<Option<serde_json::Value>, TrackerStateError> {
        let (profile, evidence, progress) = {
            let repo = self
                .repository
                .lock()
                .map_err(|_| TrackerStateError::Unavailable)?;
            let Some(profile) = requested.cloned().or(repo.active_profile()?) else {
                return Ok(None);
            };
            if !repo.profiles()?.contains(&profile) {
                return Err(TrackerStateError::InvalidProfile);
            }
            let evidence = repo
                .setting(&format!("journal_evidence_v1:{}", profile.as_str()))?
                .map(|value| serde_json::from_str(&value))
                .transpose()
                .map_err(RepoError::from)?
                .unwrap_or_default();
            (
                profile.clone(),
                evidence,
                ProfileProgress::from_events(repo.events(&profile)?),
            )
        };
        let preferences = self.preferences()?;
        let language = self.effective_language()?;
        let journal = self
            .journal
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?;
        Ok(journal.as_ref().map(|(catalog, _)| {
            let mut snapshot = crate::journal::view::snapshot(
                catalog,
                &evidence,
                &progress,
                language,
                preferences.spoiler_mode,
            );
            snapshot["profile"]["id"] = serde_json::json!(profile.as_str());
            snapshot
        }))
    }

    pub fn journal_art(&self, keys: &[String]) -> Result<serde_json::Value, TrackerStateError> {
        use base64::Engine;
        if keys.len() > 32 {
            return Err(TrackerStateError::InvalidItem);
        }
        let Some(snapshot) = self.journal_snapshot()? else {
            return Ok(serde_json::json!({}));
        };
        let journal = self
            .journal
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?;
        let Some((catalog, directory)) = journal.as_ref() else {
            return Ok(serde_json::json!({}));
        };
        let mut images = serde_json::Map::new();
        for key in keys {
            let sprite = crate::journal::view::artwork_for(catalog, &snapshot, key);
            let image = sprite
                .filter(|sprite| {
                    !sprite.is_empty()
                        && sprite
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                })
                .and_then(|sprite| std::fs::read(directory.join(format!("{sprite}.png"))).ok())
                .filter(|bytes| bytes.len() <= 4 * 1024 * 1024)
                .map(|bytes| {
                    format!(
                        "data:image/png;base64,{}",
                        base64::engine::general_purpose::STANDARD.encode(bytes)
                    )
                });
            images.insert(key.clone(), serde_json::json!(image));
        }
        Ok(serde_json::Value::Object(images))
    }

    pub fn journal_note(
        &self,
        raw_profile: &str,
        key: &str,
        text: Option<&str>,
    ) -> Result<Option<String>, TrackerStateError> {
        let profile = ProfileId::new(raw_profile).map_err(|_| TrackerStateError::InvalidProfile)?;
        let snapshot = self
            .journal_snapshot_for(Some(&profile))?
            .ok_or(TrackerStateError::NoActiveProfile)?;
        let id = snapshot["entries"]
            .as_array()
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|entry| entry["key"].as_str() == Some(key))
            })
            .and_then(|entry| entry["id"].as_str())
            .ok_or(TrackerStateError::InvalidItem)?;
        if text.is_some_and(|text| text.chars().count() > 20_000) {
            return Err(RepoError::NoteTooLong.into());
        }
        let mut repo = self
            .repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?;
        if let Ok(item) = ItemId::new(id) {
            let entity = EntityId::Item(item);
            if let Some(text) = text {
                repo.save_note(&profile, &entity, text)?;
            }
            return Ok(repo.note(&profile, &entity)?);
        }
        let storage_key = format!("journal_note:{}:{id}", profile.as_str());
        if let Some(text) = text {
            repo.save_setting(&storage_key, text)?;
        }
        Ok(repo.setting(&storage_key)?)
    }

    pub fn save_item_note(&self, raw_item_id: &str, text: &str) -> Result<(), TrackerStateError> {
        let item_id = ItemId::new(raw_item_id).map_err(|_| TrackerStateError::InvalidItem)?;
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?;
        let profile_id = repository
            .active_profile()?
            .ok_or(TrackerStateError::NoActiveProfile)?;
        repository.save_note(&profile_id, &EntityId::Item(item_id), text)?;
        Ok(())
    }

    pub fn item_note(&self, raw_item_id: &str) -> Result<Option<String>, TrackerStateError> {
        let item_id = ItemId::new(raw_item_id).map_err(|_| TrackerStateError::InvalidItem)?;
        let repository = self
            .repository
            .lock()
            .map_err(|_| TrackerStateError::Unavailable)?;
        let Some(profile_id) = repository.active_profile()? else {
            return Ok(None);
        };
        Ok(repository.note(&profile_id, &EntityId::Item(item_id))?)
    }
}

fn valid_item_sprite(sprite: &str) -> bool {
    sprite.starts_with("spr_ui_item_")
        && sprite
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compatibility::CompatibilityMatrix, persistence::Repository,
        test_support::catalog::extract_fixture_catalog,
    };

    #[test]
    fn journal_notes_remain_scoped_to_the_displayed_profile_after_activation_changes() {
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());
        let mut catalog = crate::journal::catalog::JournalCatalog::default();
        catalog.entries.insert(
            "trout".into(),
            crate::journal::catalog::Entry {
                id: "trout".into(),
                name: "Trout".into(),
                category: "fish".into(),
                ..Default::default()
            },
        );
        *state.journal.lock().unwrap() = Some((catalog, std::path::PathBuf::new()));
        for id in ["101", "202"] {
            state.activate_profile(id).unwrap();
            let mut evidence = crate::journal::evidence::JournalEvidence::default();
            evidence.donated.insert("trout".into());
            state
                .store_journal_evidence(&ProfileId::new(id).unwrap(), &evidence)
                .unwrap();
        }
        state
            .journal_note("101", "e0", Some("Ari's fishing spot"))
            .unwrap();
        assert_eq!(
            state.journal_note("101", "e0", None).unwrap().as_deref(),
            Some("Ari's fishing spot")
        );
        assert_eq!(state.journal_note("202", "e0", None).unwrap(), None);
        assert_eq!(state.active_profile().unwrap().unwrap().as_str(), "202");
        assert!(state
            .journal_note("303", "e0", Some("unregistered"))
            .is_err());
    }

    #[test]
    fn active_profile_snapshot_contains_only_progress_approved_by_spoiler_policy() {
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());
        let matrix: CompatibilityMatrix = serde_json::from_value(serde_json::json!({
            "records": [{"game_version":"synthetic-1","event_schema_versions":[1],"companion_versions":["0.1.x"],"save_parser_versions":[1],"catalog_parser_versions":[1],"probe_required":false}]
        })).unwrap();
        state
            .install_catalog(extract_fixture_catalog(), &matrix)
            .unwrap();
        state.activate_profile("1849811906").unwrap();
        let event = r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"synthetic-1","profile_id":"1849811906","session_id":"00000000-0000-0000-0000-000000000000","sequence":1,"type":"item_obtained","payload":{"item_id":"paper_pondshell","count":1}}"#;
        state.ingest_companion_event(event, &matrix).unwrap();

        let snapshot = state.active_profile_snapshot().unwrap().unwrap();
        assert_eq!(snapshot.collections.items.completed, 1);
        assert_eq!(snapshot.items.len(), 1);
        assert_eq!(snapshot.items[0].name, "Paper Pondshell");
        assert_eq!(snapshot.items[0].seasons, vec!["spring"]);
        assert_eq!(snapshot.items[0].locations, vec!["pond"]);
    }

    #[test]
    fn visible_item_icon_falls_back_to_the_safe_item_sprite_name() {
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());
        let matrix: CompatibilityMatrix = serde_json::from_value(serde_json::json!({
            "records": [{"game_version":"synthetic-1","event_schema_versions":[1],"companion_versions":["0.1.x"],"save_parser_versions":[1],"catalog_parser_versions":[1],"probe_required":false}]
        }))
        .unwrap();
        state
            .install_catalog(extract_fixture_catalog(), &matrix)
            .unwrap();
        state.activate_profile("1849811906").unwrap();
        state
            .ingest_companion_event(
                r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"synthetic-1","profile_id":"1849811906","session_id":"00000000-0000-0000-0000-000000000000","sequence":1,"type":"item_obtained","payload":{"item_id":"paper_pondshell","count":1}}"#,
                &matrix,
            )
            .unwrap();

        assert_eq!(
            state.visible_item_icon_sprite("paper_pondshell").unwrap(),
            Some("spr_ui_item_paper_pondshell".to_owned())
        );
    }

    #[test]
    fn probe_gated_catalog_cannot_be_installed_into_tracker_state() {
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());

        assert!(matches!(
            state.install_catalog(
                extract_fixture_catalog(),
                &CompatibilityMatrix::fixture_for("synthetic-1")
            ),
            Err(TrackerStateError::CatalogPaused)
        ));
        assert!(state.active_profile_snapshot().unwrap().is_none());
    }

    #[test]
    fn probe_gated_assets_archive_is_never_installed_as_a_catalog() {
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());
        let fixture = crate::test_support::catalog::fixture_assets();

        assert!(matches!(
            state.load_catalog(
                &fixture.source,
                &fixture.cache,
                &CompatibilityMatrix::fixture_for("synthetic-1")
            ),
            Err(TrackerStateError::CatalogPaused)
        ));
        assert!(state.active_profile_snapshot().unwrap().is_none());
    }

    #[test]
    fn readiness_reports_aggregate_catalog_data_without_starting_tracking() {
        let directory = tempfile::tempdir().unwrap();
        let game = directory.path().join("game");
        std::fs::create_dir(&game).unwrap();
        let fixture = crate::test_support::catalog::fixture_assets();
        std::fs::copy(fixture.source.as_path(), game.join("assets.zip")).unwrap();
        let mod_data = directory.path().join("mod_data");
        std::fs::create_dir(&mod_data).unwrap();
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());

        assert_eq!(
            state
                .readiness_report(
                    &game,
                    &mod_data,
                    &CatalogCacheDir::new(directory.path().join("cache")),
                    &CompatibilityMatrix::fixture_for("synthetic-1"),
                )
                .unwrap(),
            ReadinessReport {
                catalog: CatalogProbeReport {
                    game_version: "synthetic-1".to_owned(),
                    item_count: 1,
                },
                catalog_approved: false,
                companion_log_found: false,
            }
        );
    }
}
