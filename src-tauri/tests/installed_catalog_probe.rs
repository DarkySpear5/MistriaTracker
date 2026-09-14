use mistria_tracker_lib::{
    catalog::{AssetsZip, CatalogCacheDir, CatalogExtractor},
    domain::ItemId,
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
