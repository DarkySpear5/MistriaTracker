use mistria_tracker_lib::{
    app_state::{TrackerPreferences, TrackerState},
    compatibility::CompatibilityMatrix,
    domain::{Language, ProfileId, SpoilerMode},
    localization::LanguagePreference,
    persistence::AcceptResult,
    steam_discovery::DiscoverySources,
    tracking::ReadOnlyLogTail,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn tracker_state_scopes_an_item_note_to_the_active_local_profile() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();
    state.activate_profile("1849811906").unwrap();
    state
        .save_item_note("paper_pondshell", "Pond route")
        .unwrap();

    assert_eq!(
        state.active_profile().unwrap(),
        Some(ProfileId::new("1849811906").unwrap())
    );
    assert_eq!(
        state.item_note("paper_pondshell").unwrap(),
        Some("Pond route".to_owned())
    );
    assert_eq!(
        state.profiles().unwrap(),
        vec![ProfileId::new("1849811906").unwrap()]
    );
}

#[test]
fn tracker_state_rejects_notes_without_an_active_profile() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();

    assert!(state
        .save_item_note("paper_pondshell", "Pond route")
        .is_err());
}

#[test]
fn tracker_preferences_default_to_spoiler_free_and_normalize_hints() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();

    assert_eq!(state.preferences().unwrap(), TrackerPreferences::default());

    state
        .save_preferences(TrackerPreferences {
            language_preference: LanguagePreference::Manual(Language::Fra),
            spoiler_mode: SpoilerMode::All,
            hints_enabled: true,
            game_directory: None,
        })
        .unwrap();

    assert_eq!(
        state.preferences().unwrap(),
        TrackerPreferences {
            language_preference: LanguagePreference::Manual(Language::Fra),
            spoiler_mode: SpoilerMode::All,
            hints_enabled: false,
            game_directory: None,
        }
    );
}

#[test]
fn stale_saved_game_directory_falls_through_to_discovery() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();
    let game = directory.path().join("Fields of Mistria");
    fs::create_dir(&game).unwrap();
    fs::write(
        game.join("Maybe.toml"),
        "name = \"Fields of Mistria\"\nexecutable_name = \"FieldsOfMistria\"\n",
    )
    .unwrap();
    fs::write(game.join("assets.zip"), b"fixture assets").unwrap();
    state.save_game_directory(&game).unwrap();
    fs::remove_dir_all(&game).unwrap();

    assert_eq!(
        state
            .resolved_game_directory_from_sources(DiscoverySources::default())
            .unwrap(),
        None
    );
}

#[test]
fn legacy_language_settings_become_auto_detect_without_losing_other_preferences() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();
    let connection = rusqlite::Connection::open(directory.path().join("tracker.sqlite")).unwrap();
    connection
        .execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            (
                "tracker_preferences",
                r#"{"language":"fra","spoiler_mode":"all","hints_enabled":true}"#,
            ),
        )
        .unwrap();

    assert_eq!(
        state.preferences().unwrap(),
        TrackerPreferences {
            language_preference: LanguagePreference::Auto,
            spoiler_mode: SpoilerMode::All,
            hints_enabled: false,
            game_directory: None,
        }
    );
}

#[test]
fn companion_events_are_accepted_only_after_an_explicit_compatibility_approval() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();
    let event = r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"synthetic-1","profile_id":"1849811906","session_id":"00000000-0000-0000-0000-000000000000","sequence":1,"type":"item_obtained","payload":{"item_id":"paper_pondshell","count":1}}"#;
    let matrix: CompatibilityMatrix = serde_json::from_value(serde_json::json!({
        "records": [{"game_version":"synthetic-1","event_schema_versions":[1],"companion_versions":["0.1.x"],"save_parser_versions":[1],"catalog_parser_versions":[1],"probe_required":false}]
    })).unwrap();

    assert_eq!(
        state.ingest_companion_event(event, &matrix).unwrap(),
        AcceptResult::Inserted
    );
}

#[test]
fn companion_events_are_rejected_when_the_version_is_still_probe_gated() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();
    let event = r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"synthetic-1","profile_id":"1849811906","session_id":"00000000-0000-0000-0000-000000000000","sequence":1,"type":"item_obtained","payload":{"item_id":"paper_pondshell","count":1}}"#;

    assert!(state
        .ingest_companion_event(event, &CompatibilityMatrix::fixture_for("synthetic-1"))
        .is_err());
}

#[test]
fn irrelevant_log_lines_cannot_be_ingested_as_companion_events() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();
    let matrix: CompatibilityMatrix = serde_json::from_value(serde_json::json!({
        "records": [{"game_version":"synthetic-1","event_schema_versions":[1],"companion_versions":["0.1.x"],"save_parser_versions":[1],"catalog_parser_versions":[1],"probe_required":false}]
    })).unwrap();

    assert_eq!(
        state
            .ingest_companion_log_line("ordinary game log", &matrix)
            .unwrap(),
        None
    );
}

#[test]
fn appended_companion_events_pass_through_the_same_compatibility_gate() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();
    let log_path = directory.path().join("mmapi.log");
    fs::write(&log_path, "old history\n").unwrap();
    let mut tail = ReadOnlyLogTail::open(&log_path).unwrap();
    let event = r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"synthetic-1","profile_id":"1849811906","session_id":"00000000-0000-0000-0000-000000000000","sequence":1,"type":"item_obtained","payload":{"item_id":"paper_pondshell","count":1}}"#;
    fs::write(
        &log_path,
        format!("old history\nordinary line\nMISTRIA_TRACKER_EVENT|{event}\n"),
    )
    .unwrap();
    let matrix: CompatibilityMatrix = serde_json::from_value(serde_json::json!({
        "records": [{"game_version":"synthetic-1","event_schema_versions":[1],"companion_versions":["0.1.x"],"save_parser_versions":[1],"catalog_parser_versions":[1],"probe_required":false}]
    }))
    .unwrap();

    assert_eq!(
        state.poll_companion_log(&mut tail, &matrix).unwrap(),
        vec![AcceptResult::Inserted]
    );
}

#[test]
fn probe_gated_appended_events_are_not_accepted() {
    let directory = tempdir().unwrap();
    let state = TrackerState::open(directory.path()).unwrap();
    let log_path = directory.path().join("mmapi.log");
    fs::write(&log_path, "old history\n").unwrap();
    let mut tail = ReadOnlyLogTail::open(&log_path).unwrap();
    fs::write(
        &log_path,
        "old history\nMISTRIA_TRACKER_EVENT|{\"schema_version\":1,\"companion_version\":\"0.1.0\",\"game_version\":\"synthetic-1\",\"profile_id\":\"1849811906\",\"session_id\":\"00000000-0000-0000-0000-000000000000\",\"sequence\":1,\"type\":\"item_obtained\",\"payload\":{\"item_id\":\"paper_pondshell\",\"count\":1}}\n",
    )
    .unwrap();

    assert!(state
        .poll_companion_log(&mut tail, &CompatibilityMatrix::fixture_for("synthetic-1"))
        .is_err());
}
