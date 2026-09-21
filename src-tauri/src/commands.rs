use crate::{
    app_state::{TrackerPreferences, TrackerState, TrackerStateError},
    catalog::{cached_item_icon, CatalogCacheDir},
    compatibility::CompatibilityMatrix,
    domain::ProfileId,
    live_save::{newest_save_for_profile, save_for_companion_filename},
    persistence::ImportResult,
    safety::paths::GameAssetsPath,
    safety::paths::{CompanionLogPath, SelectedSavePath},
    safety::snapshot::{Snapshot, SnapshotService},
    save::{
        backup::{discoveries_from_bytes, profile_id_from_save_filename},
        evidence::{EvidenceExtractor, SAVE_PARSER_VERSION},
        vault::VaultReader,
    },
    tracking::PassiveTrackingSession,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Mutex};
use tauri::{Manager, State};

pub fn profile_summary(state: &TrackerState) -> Result<Value, TrackerStateError> {
    let active_profile = state
        .active_profile()?
        .map(|profile_id| profile_id.as_str().to_owned());
    let profiles = state
        .profiles()?
        .into_iter()
        .map(|profile_id| profile_id.as_str().to_owned())
        .collect::<Vec<_>>();
    Ok(json!({ "active_profile": active_profile, "profiles": profiles }))
}

pub fn preferences_value(state: &TrackerState) -> Result<Value, TrackerStateError> {
    let preferences = state.preferences()?;
    Ok(json!({
        "language_preference": preferences.language_preference,
        "effective_language": state.effective_language()?,
        "spoiler_mode": preferences.spoiler_mode,
        "hints_enabled": preferences.hints_enabled,
    }))
}

pub fn resolve_game_directory_value(
    state: &TrackerState,
) -> Result<Option<String>, TrackerStateError> {
    state
        .resolved_game_directory()
        .map(|path| path.map(|path| path.to_string_lossy().into_owned()))
}

pub fn save_game_directory_value(
    state: &TrackerState,
    game_directory: &str,
) -> Result<String, TrackerStateError> {
    state
        .save_game_directory(std::path::Path::new(game_directory))
        .map(|path| path.to_string_lossy().into_owned())
}

pub fn active_snapshot_value(state: &TrackerState) -> Result<Value, TrackerStateError> {
    match state.active_profile_snapshot()? {
        Some(snapshot) => serde_json::to_value(snapshot)
            .map_err(|error| TrackerStateError::Repository(error.into())),
        None => Ok(Value::Null),
    }
}

pub fn readiness_value(
    state: &TrackerState,
    game_directory: &str,
    mod_data_directory: &str,
) -> Result<Value, TrackerStateError> {
    let matrix = CompatibilityMatrix::embedded().map_err(crate::persistence::RepoError::from)?;
    serde_json::to_value(state.readiness_report(
        std::path::Path::new(game_directory),
        std::path::Path::new(mod_data_directory),
        &CatalogCacheDir::new(std::env::temp_dir().join("mistria-tracker-catalog-probe")),
        &matrix,
    )?)
    .map_err(|error| TrackerStateError::Repository(error.into()))
}

fn default_mod_data_directory(
    local_app_data: Option<std::ffi::OsString>,
) -> Result<PathBuf, TrackerStateError> {
    local_app_data
        .map(PathBuf::from)
        .map(|directory| directory.join("FieldsOfMistria").join("mod_data"))
        .ok_or_else(|| {
            TrackerStateError::Repository(crate::persistence::RepoError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "LOCALAPPDATA is unavailable; select the FieldsOfMistria local data directory",
            )))
        })
}

fn saves_directory_for_mod_data(
    mod_data_directory: &std::path::Path,
) -> Result<PathBuf, TrackerStateError> {
    let is_mod_data = mod_data_directory
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("mod_data"));
    let Some(local_data_root) = mod_data_directory.parent().filter(|_| is_mod_data) else {
        return Err(TrackerStateError::Repository(
            crate::persistence::RepoError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the companion log directory must be the FieldsOfMistria mod_data folder",
            )),
        ));
    };
    Ok(local_data_root.join("saves"))
}

fn default_catalog_cache_directory(
    local_app_data: Option<std::ffi::OsString>,
) -> Result<CatalogCacheDir, TrackerStateError> {
    local_app_data
        .map(PathBuf::from)
        .map(|directory| CatalogCacheDir::new(directory.join("MistriaTracker/cache/catalog")))
        .ok_or_else(|| {
            TrackerStateError::Repository(crate::persistence::RepoError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "LOCALAPPDATA is unavailable; cannot cache local game artwork",
            )))
        })
}

pub fn visible_item_icon_value(
    state: &TrackerState,
    item_id: &str,
) -> Result<Option<String>, TrackerStateError> {
    let Some(sprite_name) = state.visible_item_icon_sprite(item_id)? else {
        return Ok(None);
    };
    let cache = default_catalog_cache_directory(std::env::var_os("LOCALAPPDATA"))?;
    let Some(path) = cached_item_icon(&sprite_name, &cache)? else {
        return Ok(None);
    };
    let image = std::fs::read(path).map_err(crate::persistence::RepoError::from)?;
    Ok(Some(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(image)
    )))
}

pub(crate) struct LiveTrackingSession {
    passive: PassiveTrackingSession,
    saves_directory: PathBuf,
    tracker_local_data: PathBuf,
    imported_save_file: Option<String>,
}

pub struct LiveTrackingRuntime(pub(crate) Mutex<Option<LiveTrackingSession>>);

impl Default for LiveTrackingRuntime {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

#[tauri::command]
pub fn companion_log_available(mod_data_directory: String) -> bool {
    let root = PathBuf::from(mod_data_directory);
    CompanionLogPath::new(
        &root,
        &root.join("mistria_tracker_companion/logs/mistria_tracker_companion.log"),
    )
    .is_ok()
}

#[tauri::command]
pub fn read_journal_note(
    state: State<'_, TrackerState>,
    profile_id: String,
    key: String,
) -> Result<Option<String>, String> {
    state
        .journal_note(&profile_id, &key, None)
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn save_journal_note(
    state: State<'_, TrackerState>,
    profile_id: String,
    key: String,
    text: String,
) -> Result<(), String> {
    state
        .journal_note(&profile_id, &key, Some(&text))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_journal_snapshot(state: State<'_, TrackerState>) -> Result<Option<Value>, String> {
    state.journal_snapshot().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_journal_art(state: State<'_, TrackerState>, keys: Vec<String>) -> Result<Value, String> {
    state.journal_art(&keys).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn prepare_journal(
    state: State<'_, TrackerState>,
    game_directory: String,
) -> Result<(), String> {
    let path = std::path::Path::new(&game_directory);
    let assets = GameAssetsPath::new(path, &path.join("assets.zip")).map_err(|e| e.to_string())?;
    let cache = default_catalog_cache_directory(std::env::var_os("LOCALAPPDATA"))
        .map_err(|e| e.to_string())?;
    let matrix = CompatibilityMatrix::embedded().map_err(|e| e.to_string())?;
    state
        .load_catalog_from_game_assets(&assets, &cache, &matrix)
        .map_err(|e| e.to_string())
}

pub fn start_live_tracking_value(
    state: &TrackerState,
    runtime: &LiveTrackingRuntime,
    game_directory: &str,
    mod_data_directory: &str,
    tracker_local_data: &std::path::Path,
) -> Result<(), TrackerStateError> {
    let matrix = CompatibilityMatrix::embedded().map_err(crate::persistence::RepoError::from)?;
    let cache = default_catalog_cache_directory(std::env::var_os("LOCALAPPDATA"))?;
    start_live_tracking_value_with_compatibility(
        state,
        runtime,
        game_directory,
        mod_data_directory,
        &saves_directory_for_mod_data(std::path::Path::new(mod_data_directory))?,
        tracker_local_data,
        &cache,
        &matrix,
    )
}

fn start_live_tracking_value_with_compatibility(
    state: &TrackerState,
    runtime: &LiveTrackingRuntime,
    game_directory: &str,
    mod_data_directory: &str,
    saves_directory: &std::path::Path,
    tracker_local_data: &std::path::Path,
    cache: &CatalogCacheDir,
    matrix: &CompatibilityMatrix,
) -> Result<(), TrackerStateError> {
    let game_directory = std::path::Path::new(game_directory);
    let assets = GameAssetsPath::new(game_directory, &game_directory.join("assets.zip"))?;
    let mod_data_directory = std::path::Path::new(mod_data_directory);
    let log = CompanionLogPath::new(
        mod_data_directory,
        &mod_data_directory.join("mistria_tracker_companion/logs/mistria_tracker_companion.log"),
    )?;
    let session =
        PassiveTrackingSession::start(state, &assets, &log, cache, matrix).map_err(|error| {
            TrackerStateError::Repository(crate::persistence::RepoError::Io(std::io::Error::other(
                error.to_string(),
            )))
        })?;
    *runtime
        .0
        .lock()
        .map_err(|_| TrackerStateError::Unavailable)? = Some(LiveTrackingSession {
        passive: session,
        saves_directory: saves_directory.to_path_buf(),
        tracker_local_data: tracker_local_data.to_path_buf(),
        imported_save_file: None,
    });
    Ok(())
}

pub fn poll_live_tracking_value(
    state: &TrackerState,
    runtime: &LiveTrackingRuntime,
) -> Result<usize, TrackerStateError> {
    let matrix = CompatibilityMatrix::embedded().map_err(crate::persistence::RepoError::from)?;
    poll_live_tracking_value_with_compatibility(state, runtime, &matrix)
}

fn poll_live_tracking_value_with_compatibility(
    state: &TrackerState,
    runtime: &LiveTrackingRuntime,
    matrix: &CompatibilityMatrix,
) -> Result<usize, TrackerStateError> {
    let mut runtime = runtime
        .0
        .lock()
        .map_err(|_| TrackerStateError::Unavailable)?;
    let session = runtime
        .as_mut()
        .ok_or(TrackerStateError::LiveTrackingPaused)?;
    let accepted = session.passive.poll(state, matrix).map_err(|error| {
        TrackerStateError::Repository(crate::persistence::RepoError::Io(std::io::Error::other(
            error.to_string(),
        )))
    })?;
    if !accepted.is_empty() {
        if let (Some(profile_id), Some(save_file)) =
            (state.active_profile()?, state.active_save_file()?)
        {
            if session.imported_save_file.as_deref() != Some(save_file.as_str()) {
                let imported = import_confirmed_live_save_value(
                    state,
                    &profile_id,
                    Some(&save_file),
                    &session.saves_directory,
                    &session.tracker_local_data,
                    matrix,
                )?;
                if imported.is_some() {
                    session.imported_save_file = Some(save_file);
                }
            }
        }
    }
    Ok(accepted.len())
}

/// Imports one save file explicitly selected by the player. The original is
/// canonicalized and read only; parsing happens only against a temporary copy
/// owned by the tracker.
pub fn import_selected_save_value(
    state: &TrackerState,
    save_path: &std::path::Path,
    tracker_local_data: &std::path::Path,
) -> Result<Value, TrackerStateError> {
    let selected = SelectedSavePath::new(save_path)?;
    let profile_id = profile_id_from_save_filename(selected.as_path()).map_err(|error| {
        TrackerStateError::Repository(crate::persistence::RepoError::Io(std::io::Error::other(
            error.to_string(),
        )))
    })?;
    let compatibility =
        CompatibilityMatrix::embedded().map_err(crate::persistence::RepoError::from)?;
    let snapshot = SnapshotService::for_local_data(tracker_local_data)
        .and_then(|service| service.snapshot_selected(&selected))
        .map_err(|error| {
            TrackerStateError::Repository(crate::persistence::RepoError::Io(std::io::Error::other(
                error.to_string(),
            )))
        })?;
    import_snapshot_value(state, snapshot, profile_id, &compatibility)
}

/// Imports the exact save file reported by the companion for the confirmed
/// profile. The profile-only fallback is retained for explicit probes and
/// older callers, but it never considers a save from another profile.
pub fn import_confirmed_live_save_value(
    state: &TrackerState,
    profile_id: &ProfileId,
    save_file: Option<&str>,
    saves_directory: &std::path::Path,
    tracker_local_data: &std::path::Path,
    compatibility: &CompatibilityMatrix,
) -> Result<Option<Value>, TrackerStateError> {
    let save = match save_file {
        Some(save_file) => Some(
            save_for_companion_filename(saves_directory, profile_id, save_file).map_err(
                |error| {
                    TrackerStateError::Repository(crate::persistence::RepoError::Io(
                        std::io::Error::other(error.to_string()),
                    ))
                },
            )?,
        ),
        None => newest_save_for_profile(saves_directory, profile_id).map_err(|error| {
            TrackerStateError::Repository(crate::persistence::RepoError::Io(std::io::Error::other(
                error.to_string(),
            )))
        })?,
    };
    let Some(save) = save else {
        return Ok(None);
    };
    let snapshot = SnapshotService::for_local_data(tracker_local_data)
        .and_then(|service| service.snapshot(&save))
        .map_err(|error| {
            TrackerStateError::Repository(crate::persistence::RepoError::Io(std::io::Error::other(
                error.to_string(),
            )))
        })?;
    import_snapshot_value(state, snapshot, profile_id.clone(), compatibility).map(Some)
}

fn import_snapshot_value(
    state: &TrackerState,
    snapshot: Snapshot,
    profile_id: ProfileId,
    compatibility: &CompatibilityMatrix,
) -> Result<Value, TrackerStateError> {
    let result = (|| {
        let bytes = std::fs::read(&snapshot.path).map_err(crate::persistence::RepoError::from)?;
        let vault = VaultReader::read(bytes.as_slice()).map_err(|error| {
            TrackerStateError::Repository(crate::persistence::RepoError::Io(std::io::Error::other(
                error.to_string(),
            )))
        })?;
        let game_version = EvidenceExtractor::game_version(&vault).map_err(|error| {
            TrackerStateError::Repository(crate::persistence::RepoError::Io(std::io::Error::other(
                error.to_string(),
            )))
        })?;
        if compatibility.save_parser_decision(&game_version, SAVE_PARSER_VERSION)
            != crate::compatibility::SaveParserDecision::Verified
        {
            return Ok(json!({
                "status": "unsupported_version",
                "profile_id": profile_id.as_str(),
                "game_version": game_version,
                "discovered_items": 0,
                "imported": false,
            }));
        }
        let discoveries =
            discoveries_from_bytes(&bytes, profile_id, compatibility).map_err(|error| {
                TrackerStateError::Repository(crate::persistence::RepoError::Io(
                    std::io::Error::other(error.to_string()),
                ))
            })?;
        let imported = state.import_existing_items(
            discoveries.source_hash,
            &discoveries.profile_id,
            &discoveries.game_version,
            &discoveries.items,
        )?;
        state.store_journal_evidence(&discoveries.profile_id, &discoveries.journal)?;
        Ok(json!({
            "status": if matches!(imported, ImportResult::Inserted) {
                "imported"
            } else {
                "already_imported"
            },
            "profile_id": discoveries.profile_id.as_str(),
            "discovered_items": discoveries.items.len(),
            "imported": matches!(imported, ImportResult::Inserted),
        }))
    })();
    let cleanup = std::fs::remove_file(&snapshot.path)
        .and_then(|_| std::fs::remove_file(&snapshot.manifest_path));
    if let Err(error) = cleanup {
        return Err(TrackerStateError::Repository(
            crate::persistence::RepoError::Io(error),
        ));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compatibility::CompatibilityMatrix,
        persistence::Repository,
        test_support::{
            catalog::{extract_fixture_catalog, fixture_assets},
            vault::{vault_bytes, Fixture},
        },
    };
    use std::fs;

    #[test]
    fn derives_the_mod_data_and_sibling_saves_paths_from_local_app_data() {
        let local = std::path::PathBuf::from(r"C:\Users\Ari\AppData\Local");
        let mod_data = local.join("FieldsOfMistria").join("mod_data");
        assert_eq!(
            default_mod_data_directory(Some(std::ffi::OsString::from(
                r"C:\Users\Ari\AppData\Local"
            )))
            .unwrap(),
            mod_data,
        );
        assert_eq!(
            saves_directory_for_mod_data(&mod_data).unwrap(),
            local.join("FieldsOfMistria").join("saves"),
        );
    }

    #[test]
    fn active_snapshot_value_returns_the_spoiler_filtered_desktop_view() {
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
            active_snapshot_value(&state).unwrap()["items"][0]["name"],
            "Paper Pondshell"
        );
    }

    #[test]
    fn imports_only_a_stable_tracker_snapshot_of_the_explicitly_selected_save() {
        let fixture = Fixture::new();
        let selected_save = fixture.local.join("Ari-game-331655283-42.sav");
        fs::write(
            &selected_save,
            vault_bytes(&[
                ("info", serde_json::json!({"version":"1.0.4"})),
                (
                    "header",
                    serde_json::json!({"name":"Ari","farm_name":"Test"}),
                ),
                ("player", serde_json::json!({"items_acquired":["test_ore"]})),
            ]),
        )
        .unwrap();
        let before = fs::read(&selected_save).unwrap();
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());

        let result = import_selected_save_value(&state, &selected_save, &fixture.local).unwrap();

        assert_eq!(result["profile_id"], "331655283");
        assert_eq!(result["discovered_items"], 1);
        assert_eq!(result["status"], "imported");
        assert_eq!(fs::read(&selected_save).unwrap(), before);
        assert_eq!(
            state.active_profile().unwrap().unwrap().as_str(),
            "331655283"
        );
        let snapshots = fixture.local.join("MistriaTracker/backups/game-saves");
        assert_eq!(fs::read_dir(snapshots).unwrap().count(), 0);
    }

    #[test]
    fn refuses_unknown_save_versions_without_writing_tracker_state() {
        let fixture = Fixture::new();
        let selected_save = fixture.local.join("Ari-game-331655283-42.sav");
        fs::write(
            &selected_save,
            vault_bytes(&[
                ("info", serde_json::json!({"version":"9.9.9"})),
                (
                    "header",
                    serde_json::json!({"name":"Ari","farm_name":"Test"}),
                ),
            ]),
        )
        .unwrap();
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());
        let report = import_selected_save_value(&state, &selected_save, &fixture.local).unwrap();
        assert_eq!(report["status"], "unsupported_version");
        assert_eq!(report["game_version"], "9.9.9");
        assert!(state.active_profile().unwrap().is_none());
    }

    #[test]
    fn removes_the_tracker_snapshot_when_selected_save_parsing_fails() {
        let fixture = Fixture::new();
        let selected_save = fixture.local.join("Ari-game-331655283-42.sav");
        fs::write(&selected_save, b"not a Fields of Mistria save").unwrap();
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());

        assert!(import_selected_save_value(&state, &selected_save, &fixture.local).is_err());
        let snapshots = fixture.local.join("MistriaTracker/backups/game-saves");
        assert_eq!(fs::read_dir(snapshots).unwrap().count(), 0);
    }

    #[test]
    fn confirmed_live_profile_imports_only_its_1_0_5_save_and_exposes_the_character_name() {
        let fixture = Fixture::new();
        let matching = fixture.saves.join("game-331655283-42.sav");
        fs::write(
            &matching,
            vault_bytes(&[
                (
                    "info",
                    serde_json::json!({"version":{"major":1,"minor":0,"patch":5,"pre":null}}),
                ),
                (
                    "header",
                    serde_json::json!({"name":"Ari","farm_name":"Test Farm"}),
                ),
                ("player", serde_json::json!({"items_acquired":["test_ore"]})),
            ]),
        )
        .unwrap();
        let before = fs::read(&matching).unwrap();
        fs::write(
            fixture.saves.join("game-1849811906-999.sav"),
            vault_bytes(&[
                (
                    "info",
                    serde_json::json!({"version":{"major":1,"minor":0,"patch":5,"pre":null}}),
                ),
                (
                    "header",
                    serde_json::json!({"name":"Wrong profile","farm_name":"Wrong farm"}),
                ),
                ("player", serde_json::json!({"items_acquired":[]})),
            ]),
        )
        .unwrap();
        let matrix: CompatibilityMatrix = serde_json::from_value(serde_json::json!({
            "records": [
                {
                    "game_version":"synthetic-1",
                    "event_schema_versions":[1],
                    "companion_versions":["0.1.x"],
                    "save_parser_versions":[1],
                    "catalog_parser_versions":[1],
                    "probe_required":false
                },
                {
                    "game_version":"1.0.5",
                    "event_schema_versions":[1],
                    "companion_versions":["0.1.x"],
                    "save_parser_versions":[1],
                    "catalog_parser_versions":[],
                    "probe_required":false
                }
            ]
        }))
        .unwrap();
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());
        let catalog = fixture_assets();
        state
            .load_catalog(&catalog.source, &catalog.cache, &matrix)
            .unwrap();

        let report = import_confirmed_live_save_value(
            &state,
            &ProfileId::new("331655283").unwrap(),
            Some("game-331655283-42.sav"),
            &fixture.saves,
            &fixture.local,
            &matrix,
        )
        .unwrap()
        .unwrap();

        assert_eq!(report["status"], "imported");
        assert_eq!(report["profile_id"], "331655283");
        assert_eq!(fs::read(&matching).unwrap(), before);
        assert_eq!(
            state.journal_snapshot().unwrap().unwrap()["profile"],
            serde_json::json!({"id":"331655283","name":"Ari","farm":"Test Farm"})
        );
        let snapshots = fixture.local.join("MistriaTracker/backups/game-saves");
        assert_eq!(fs::read_dir(snapshots).unwrap().count(), 0);
    }

    #[test]
    fn live_poll_imports_the_save_for_a_newly_activated_profile() {
        let fixture = Fixture::new();
        let game = fixture.local.join("game");
        fs::create_dir(&game).unwrap();
        let catalog = fixture_assets();
        fs::copy(catalog.source.as_path(), game.join("assets.zip")).unwrap();
        let mod_data = fixture.local.join("FieldsOfMistria");
        let logs = mod_data.join("mistria_tracker_companion/logs");
        fs::create_dir_all(&logs).unwrap();
        let log = logs.join("mistria_tracker_companion.log");
        fs::write(&log, "old history\n").unwrap();
        fs::write(
            fixture.saves.join("game-331655283-42.sav"),
            vault_bytes(&[
                (
                    "info",
                    serde_json::json!({"version":{"major":1,"minor":0,"patch":5,"pre":null}}),
                ),
                (
                    "header",
                    serde_json::json!({"name":"Ari","farm_name":"Test Farm"}),
                ),
                ("player", serde_json::json!({"items_acquired":["test_ore"]})),
            ]),
        )
        .unwrap();
        fs::write(
            fixture.saves.join("game-331655283-999.sav"),
            vault_bytes(&[
                (
                    "info",
                    serde_json::json!({"version":{"major":1,"minor":0,"patch":5,"pre":null}}),
                ),
                (
                    "header",
                    serde_json::json!({"name":"Wrong slot","farm_name":"Wrong farm"}),
                ),
                ("player", serde_json::json!({"items_acquired":[]})),
            ]),
        )
        .unwrap();
        let matrix: CompatibilityMatrix = serde_json::from_value(serde_json::json!({
            "records": [
                {
                    "game_version":"synthetic-1",
                    "event_schema_versions":[1],
                    "companion_versions":["0.1.x"],
                    "save_parser_versions":[1],
                    "catalog_parser_versions":[1],
                    "probe_required":false
                },
                {
                    "game_version":"1.0.5",
                    "event_schema_versions":[1],
                    "companion_versions":["0.1.x"],
                    "save_parser_versions":[1],
                    "catalog_parser_versions":[],
                    "probe_required":false
                }
            ]
        }))
        .unwrap();
        let state = TrackerState::from_repository(Repository::in_memory().unwrap());
        let runtime = LiveTrackingRuntime(Mutex::new(None));
        start_live_tracking_value_with_compatibility(
            &state,
            &runtime,
            game.to_string_lossy().as_ref(),
            mod_data.to_string_lossy().as_ref(),
            &fixture.saves,
            &fixture.local,
            &catalog.cache,
            &matrix,
        )
        .unwrap();
        fs::write(
            &log,
            concat!(
                "old history\n",
                "MISTRIA_TRACKER_EVENT|{\"schema_version\":1,\"companion_version\":\"0.1.5\",\"game_version\":\"1.0.5\",\"profile_id\":\"331655283\",\"session_id\":\"00000000-0000-7000-8000-000000000001\",\"sequence\":1,\"type\":\"profile_activated\",\"save_file\":\"game-331655283-42.sav\"}\n"
            ),
        )
        .unwrap();

        assert_eq!(
            poll_live_tracking_value_with_compatibility(&state, &runtime, &matrix).unwrap(),
            1
        );
        assert_eq!(
            state.journal_snapshot().unwrap().unwrap()["profile"],
            serde_json::json!({"id":"331655283","name":"Ari","farm":"Test Farm"})
        );
    }
}

#[tauri::command]
pub fn get_profile_summary(state: State<'_, TrackerState>) -> Result<Value, String> {
    profile_summary(&state).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn select_profile(state: State<'_, TrackerState>, profile_id: String) -> Result<(), String> {
    state
        .activate_profile(&profile_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_preferences(state: State<'_, TrackerState>) -> Result<Value, String> {
    preferences_value(&state).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn resolve_game_directory(state: State<'_, TrackerState>) -> Result<Option<String>, String> {
    resolve_game_directory_value(&state).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_game_directory(
    state: State<'_, TrackerState>,
    game_directory: String,
) -> Result<String, String> {
    save_game_directory_value(&state, &game_directory).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_active_snapshot(state: State<'_, TrackerState>) -> Result<Value, String> {
    active_snapshot_value(&state).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn probe_readiness(
    state: State<'_, TrackerState>,
    game_directory: String,
    mod_data_directory: Option<String>,
) -> Result<Value, String> {
    let mod_data_directory = match mod_data_directory {
        Some(directory) => PathBuf::from(directory),
        None => default_mod_data_directory(std::env::var_os("LOCALAPPDATA"))
            .map_err(|error| error.to_string())?,
    };
    readiness_value(
        &state,
        &game_directory,
        mod_data_directory.to_string_lossy().as_ref(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_preferences(
    state: State<'_, TrackerState>,
    preferences: TrackerPreferences,
) -> Result<(), String> {
    state
        .save_preferences(preferences)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn read_item_note(
    state: State<'_, TrackerState>,
    item_id: String,
) -> Result<Option<String>, String> {
    state.item_note(&item_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_item_note(
    state: State<'_, TrackerState>,
    item_id: String,
    text: String,
) -> Result<(), String> {
    state
        .save_item_note(&item_id, &text)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_visible_item_icon(
    state: State<'_, TrackerState>,
    item_id: String,
) -> Result<Option<String>, String> {
    visible_item_icon_value(&state, &item_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn start_live_tracking(
    state: State<'_, TrackerState>,
    runtime: State<'_, LiveTrackingRuntime>,
    app: tauri::AppHandle,
    game_directory: String,
    mod_data_directory: Option<String>,
) -> Result<(), String> {
    let mod_data_directory = match mod_data_directory {
        Some(directory) => directory,
        None => default_mod_data_directory(std::env::var_os("LOCALAPPDATA"))
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .into_owned(),
    };
    let tracker_local_data = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    start_live_tracking_value(
        &state,
        &runtime,
        &game_directory,
        &mod_data_directory,
        &tracker_local_data,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn poll_live_tracking(
    state: State<'_, TrackerState>,
    runtime: State<'_, LiveTrackingRuntime>,
) -> Result<usize, String> {
    poll_live_tracking_value(&state, &runtime).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_selected_save(
    state: State<'_, TrackerState>,
    app: tauri::AppHandle,
    save_path: String,
) -> Result<Value, String> {
    let tracker_local_data = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    import_selected_save_value(
        &state,
        std::path::Path::new(&save_path),
        &tracker_local_data,
    )
    .map_err(|error| error.to_string())
}
