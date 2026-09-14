use base64::Engine;
use mistria_tracker_lib::{
    catalog::{AssetsZip, CatalogCacheDir},
    compatibility::CompatibilityMatrix,
    domain::{Language, SpoilerMode},
    journal::{art, catalog::JournalCatalog, view},
    save::backup::existing_discoveries,
    tracking::ProfileProgress,
};
use std::{collections::BTreeMap, fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let source = AssetsZip::new(&args[1])?;
    let backup = existing_discoveries(
        std::path::Path::new(&args[2]),
        &CompatibilityMatrix::embedded()?,
    )?;
    let root = PathBuf::from(&args[3]);
    fs::create_dir_all(&root)?;
    let catalog = JournalCatalog::extract(&source)?;
    let cache = art::prepare(&source, &CatalogCacheDir::new(root.join("cache")), &catalog)?;
    let snapshot = view::snapshot(
        &catalog,
        &backup.journal,
        &ProfileProgress::from_discovered(backup.items),
        Language::Fra,
        SpoilerMode::Free,
    );
    fs::write(root.join("journal.json"), serde_json::to_vec(&snapshot)?)?;
    let mut images = BTreeMap::new();
    let mut tokens = Vec::new();
    for entry in snapshot["entries"].as_array().unwrap() {
        if let Some(key) = entry["art"].as_str() {
            tokens.push(key.to_owned());
        }
    }
    for npc in snapshot["villagers"].as_array().unwrap() {
        if let Some(key) = npc["art"].as_str() {
            tokens.push(key.to_owned());
        }
        for group in ["loved", "liked"] {
            for gift in npc[group].as_array().unwrap() {
                if let Some(key) = gift["art"].as_str() {
                    tokens.push(key.to_owned());
                }
            }
        }
    }
    for key in tokens {
        if let Some(sprite) = view::artwork_for(&catalog, &snapshot, &key) {
            if let Ok(bytes) = fs::read(cache.join(format!("{sprite}.png"))) {
                images.insert(
                    key,
                    format!(
                        "data:image/png;base64,{}",
                        base64::engine::general_purpose::STANDARD.encode(bytes)
                    ),
                );
            }
        }
    }
    fs::write(root.join("art.json"), serde_json::to_vec(&images)?)?;
    println!(
        "entries={} sets={} villagers={} rendered_art={}",
        catalog.entries.len(),
        catalog.sets.len(),
        catalog.villagers.len(),
        images.len()
    );
    println!("categories={}", snapshot["categories"]);
    Ok(())
}
