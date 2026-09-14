use mistria_tracker_lib::{compatibility::CompatibilityMatrix, save::backup::existing_discoveries};
use std::{env, path::Path};

#[test]
#[ignore = "requires MISTRIA_TRACKER_BACKUP to point to a user-approved backup copy"]
fn desktop_backup_yields_existing_item_discoveries_without_writing_it() {
    let backup = env::var("MISTRIA_TRACKER_BACKUP")
        .expect("set MISTRIA_TRACKER_BACKUP to a user-approved backup copy");
    let compatibility: CompatibilityMatrix = serde_json::from_value(serde_json::json!({
        "records": [{
            "game_version": "1.0.4",
            "event_schema_versions": [1],
            "companion_versions": ["0.1.x"],
            "save_parser_versions": [1],
            "catalog_parser_versions": [1],
            "probe_required": false
        }]
    }))
    .expect("local approved parser matrix");
    let discoveries = existing_discoveries(Path::new(&backup), &compatibility)
        .expect("extract only approved existing discovery fields");

    println!(
        "verified_save_version=1.0.4; unique_existing_items={}",
        discoveries.items.len()
    );
    assert!(
        !discoveries.items.is_empty(),
        "approved backup has no item discovery evidence"
    );
}
