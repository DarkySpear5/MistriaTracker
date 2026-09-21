use mistria_tracker_lib::{
    app_state::{TrackerPreferences, TrackerState},
    commands::{
        import_confirmed_live_save_value, import_selected_save_value, preferences_value,
        profile_summary, save_game_directory_value,
    },
    compatibility::CompatibilityMatrix,
    domain::{Language, ProfileId, SpoilerMode},
    localization::LanguagePreference,
};
use std::{fs, io::Write};
use tempfile::tempdir;

#[test]
fn profile_summary_exposes_only_tracker_owned_profile_identifiers() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();
    state.activate_profile("1849811906").unwrap();

    assert_eq!(
        profile_summary(&state).unwrap(),
        serde_json::json!({
            "active_profile": "1849811906",
            "profiles": ["1849811906"]
        })
    );
}

#[test]
fn profile_summary_has_no_active_profile_for_a_new_tracker_database() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();

    assert_eq!(
        profile_summary(&state).unwrap(),
        serde_json::json!({"active_profile": null, "profiles": []})
    );
}

#[test]
fn preferences_value_uses_safe_normalized_tracker_preferences() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();
    state
        .save_preferences(TrackerPreferences {
            language_preference: LanguagePreference::Manual(Language::Fra),
            spoiler_mode: SpoilerMode::Free,
            hints_enabled: true,
            game_directory: None,
        })
        .unwrap();

    assert_eq!(
        preferences_value(&state).unwrap(),
        serde_json::json!({
            "language_preference": "fra",
            "effective_language": "fra",
            "spoiler_mode": "free",
            "hints_enabled": true
        })
    );
}

#[test]
fn save_game_directory_rejects_a_folder_without_assets_zip() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();

    assert!(
        save_game_directory_value(&state, directory.path().to_string_lossy().as_ref()).is_err()
    );
}

#[test]
fn selected_save_import_only_reads_the_explicit_copy_and_reports_its_item_count() {
    let directory = tempdir().unwrap();
    let selected = directory.path().join("Amelia-game-1849811906-1.sav");
    fs::write(&selected, v1_0_4_vault()).unwrap();
    let state = TrackerState::open(directory.path()).unwrap();

    assert_eq!(
        import_selected_save_value(&state, &selected, directory.path()).unwrap(),
        serde_json::json!({
            "status": "imported",
            "profile_id": "1849811906",
            "discovered_items": 2,
            "imported": true,
        })
    );
    assert_eq!(
        state.active_profile().unwrap().unwrap().as_str(),
        "1849811906"
    );
}

#[test]
#[ignore = "requires MISTRIA_TRACKER_SAVES_DIR and MISTRIA_TRACKER_PROFILE_ID"]
fn installed_1_0_5_save_parses_only_from_a_stable_tracker_owned_snapshot() {
    let saves = std::env::var_os("MISTRIA_TRACKER_SAVES_DIR")
        .map(std::path::PathBuf::from)
        .expect("MISTRIA_TRACKER_SAVES_DIR is required");
    let profile = std::env::var("MISTRIA_TRACKER_PROFILE_ID")
        .expect("MISTRIA_TRACKER_PROFILE_ID is required");
    let profile = ProfileId::new(profile).expect("profile id must be numeric");
    let tracker_data = tempdir().unwrap();
    let state = TrackerState::open(tracker_data.path()).unwrap();

    let report = import_confirmed_live_save_value(
        &state,
        &profile,
        None,
        &saves,
        tracker_data.path(),
        &CompatibilityMatrix::embedded().unwrap(),
    )
    .unwrap()
    .expect("the confirmed profile must have a save");

    assert_eq!(report["status"], "imported");
    assert_eq!(report["profile_id"], profile.as_str());
    assert!(report["discovered_items"].as_u64().unwrap_or_default() > 0);
    let snapshots = tracker_data
        .path()
        .join("MistriaTracker/backups/game-saves");
    assert_eq!(fs::read_dir(snapshots).unwrap().count(), 0);
}

fn v1_0_4_vault() -> Vec<u8> {
    let parts = [
        (
            "info",
            serde_json::json!({"version":{"major":1,"minor":0,"patch":4,"pre":null}}),
        ),
        (
            "header",
            serde_json::json!({"name":"Ari","farm_name":"Test"}),
        ),
        (
            "player",
            serde_json::json!({"items_acquired":["test_ore","test_seed"]}),
        ),
    ];
    let mut raw = (parts.len() as u64).to_le_bytes().to_vec();
    for (name, payload) in parts {
        let payload = serde_json::to_vec(&payload).unwrap();
        raw.extend_from_slice(&(name.len() as u64).to_le_bytes());
        raw.extend_from_slice(name.as_bytes());
        raw.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        raw.extend_from_slice(&payload);
    }
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&raw).unwrap();
    encoder.finish().unwrap()
}
