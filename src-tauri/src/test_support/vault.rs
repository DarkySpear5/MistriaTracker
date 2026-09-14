//! Entirely invented fixtures; no values were collected from a player or game file.
use flate2::{write::ZlibEncoder, Compression};
use serde_json::{json, Value};
use std::{fs, io::Write, path::PathBuf};
use tempfile::TempDir;

pub fn compress(raw: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(raw).unwrap();
    encoder.finish().unwrap()
}

pub fn vault_bytes(sections: &[(&str, Value)]) -> Vec<u8> {
    let mut raw = (sections.len() as u64).to_le_bytes().to_vec();
    for (name, payload) in sections {
        let payload = serde_json::to_vec(payload).unwrap();
        raw.extend_from_slice(&(name.len() as u64).to_le_bytes());
        raw.extend_from_slice(name.as_bytes());
        raw.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        raw.extend_from_slice(&payload);
    }
    compress(&raw)
}

pub fn sections() -> Vec<(&'static str, Value)> {
    vec![
        ("info", json!({"version":"synthetic-1"})),
        ("header", json!({"name":"Ari", "farm_name":"Test"})),
        (
            "player",
            json!({
                "items_acquired":{"test_ore":2,"unowned_item":0},
                "inventory":[{"item_id":"test_seed","count":3},null],
                "legendary_fish_caught":{"test_legendary":true,"uncaught_fish":false},
                "story_progress":{"secret_item":true}
            }),
        ),
        (
            "npcs",
            json!({
                "test_npc":{"had_arrived":true,"gifts_given":{"test_seed":1},
                    "known_gift_preferences":{"test_ore":"loved"}},
                "unknown_npc":{"had_arrived":false,"relationship_level":10}
            }),
        ),
        (
            "gamedata",
            json!({"museum_progress":{"test_set":{"test_ore":true,"undonated_item":false}}}),
        ),
        (
            "game_stats",
            json!({"fish_caught":4,"bugs_caught":2,"gifts_given":1}),
        ),
    ]
}

pub struct Fixture {
    pub _temp: TempDir,
    pub saves: PathBuf,
    pub source: PathBuf,
    pub local: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        // Temp files are deliberately inside the tracker worktree, never AppData.
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/synthetic-fixtures");
        fs::create_dir_all(&base).unwrap();
        let temp = tempfile::tempdir_in(base).unwrap();
        let saves = temp.path().join("saves");
        let local = temp.path().join("local");
        fs::create_dir(&saves).unwrap();
        fs::create_dir(&local).unwrap();
        let source = saves.join("123-autosave.sav");
        fs::write(&source, vault_bytes(&sections())).unwrap();
        Self {
            _temp: temp,
            saves,
            source,
            local,
        }
    }
}

pub fn verified_matrix() -> crate::compatibility::matrix::CompatibilityMatrix {
    serde_json::from_value(json!({"records":[{
        "game_version":"synthetic-1", "event_schema_versions":[1],
        "companion_versions":["0.1.x"], "save_parser_versions":[1],
        "catalog_parser_versions":[1], "probe_required":false
    }]}))
    .unwrap()
}
