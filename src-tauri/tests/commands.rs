use mistria_tracker_lib::{
    app_state::{TrackerPreferences, TrackerState},
    commands::{import_latest_desktop_backup_value, preferences_value, profile_summary},
    domain::{Language, SpoilerMode},
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
            language: Language::Fra,
            spoiler_mode: SpoilerMode::Free,
            hints_enabled: true,
        })
        .unwrap();

    assert_eq!(
        preferences_value(&state).unwrap(),
        serde_json::json!({
            "language": "fra",
            "spoiler_mode": "free",
            "hints_enabled": true
        })
    );
}

#[test]
fn desktop_backup_import_only_reads_the_latest_copy_and_reports_its_item_count() {
    let directory = tempdir().unwrap();
    let backup_directory = directory.path().join("Mistria Save Backups");
    fs::create_dir(&backup_directory).unwrap();
    let backup = backup_directory.join("Amelia-game-1849811906-1.sav");
    fs::write(&backup, v1_0_4_vault()).unwrap();
    let state = TrackerState::open(directory.path()).unwrap();

    assert_eq!(
        import_latest_desktop_backup_value(&state, &[backup_directory]).unwrap(),
        serde_json::json!({
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
