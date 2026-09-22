use mistria_tracker_lib::{
    catalog::{AssetsZip, CatalogCacheDir, CatalogExtractor},
    domain::ItemId,
    journal::catalog::JournalCatalog,
};
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
#[ignore = "requires MISTRIA_TRACKER_ASSETS_ZIP to point to a read-only game assets archive"]
fn installed_assets_archive_has_an_explicit_catalog_fingerprint() {
    let path = PathBuf::from(
        std::env::var("MISTRIA_TRACKER_ASSETS_ZIP")
            .expect("set MISTRIA_TRACKER_ASSETS_ZIP to an assets.zip path"),
    );
    let temporary = tempdir().unwrap();
    let catalog = CatalogExtractor::extract(
        &AssetsZip::new(path).unwrap(),
        &CatalogCacheDir::new(temporary.path().join("catalog-cache")),
    )
    .unwrap();

    println!("{}", catalog.version.source_game_version);
    assert!(catalog
        .version
        .source_game_version
        .starts_with("catalog-sha256:"));
    assert!(catalog.items().count() > 100);
    assert_eq!(
        catalog
            .item(&ItemId::new("paper_pondshell").unwrap())
            .unwrap()
            .text("fra")
            .name,
        "Anodonte papyracée"
    );
}

#[test]
#[ignore = "requires MISTRIA_TRACKER_ASSETS_ZIP to point to a read-only game assets archive"]
fn installed_assets_archive_builds_the_full_journal_catalog() {
    let path = PathBuf::from(
        std::env::var("MISTRIA_TRACKER_ASSETS_ZIP")
            .expect("set MISTRIA_TRACKER_ASSETS_ZIP to an assets.zip path"),
    );
    let journal = JournalCatalog::extract(&AssetsZip::new(path).unwrap()).unwrap();

    println!(
        "entries={} villagers={} museum_sets={}",
        journal.entries.len(),
        journal.villagers.len(),
        journal.sets.len()
    );
    assert!(journal.entries.len() > 100);
    assert!(journal.entries.contains_key("paper_pondshell"));
    for category in [
        "fish",
        "bugs",
        "crops",
        "forageables",
        "artifacts",
        "recipes",
    ] {
        let hinted = journal
            .entries
            .values()
            .filter(|entry| {
                entry.category == category
                    && (!entry.hint_facts.areas.is_empty()
                        || !entry.hint_facts.seasons.is_empty()
                        || entry.hint_facts.recipe_source.is_some())
            })
            .count();
        println!("{category} data-backed hints={hinted}");
        assert!(hinted > 0, "no data-backed hints for {category}");
    }
}
