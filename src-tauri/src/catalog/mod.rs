pub mod availability;
pub mod extract;
pub mod icons;
pub mod items;
pub mod localization;
pub mod probe;

pub use availability::LocationTag;
pub use extract::{AssetsZip, CatalogCacheDir, CatalogError, CatalogExtractor};
pub use icons::{cached_item_icon, prepare_item_icon_cache, IconCacheReport};
pub use items::{Catalog, CatalogItem, CatalogVersion};
pub use probe::{probe, CatalogProbeReport};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::ItemId,
        test_support::catalog::{extract_fixture_catalog, extract_reordered_fixture},
    };

    #[test]
    fn extracts_item_and_french_translation_with_english_fallback() {
        let catalog = extract_fixture_catalog();
        let item = catalog
            .item(&ItemId::new("paper_pondshell").unwrap())
            .unwrap();
        assert_eq!(item.text("fra").name, "Coquille de papier");
        assert_eq!(item.text("fra").description, item.text("eng").description);
        assert_eq!(item.seasons, vec!["spring"]);
        assert!(item.locations.contains(&LocationTag::Pond));
    }

    #[test]
    fn numeric_ids_never_become_identity() {
        assert_eq!(
            extract_reordered_fixture().item_ids(),
            vec![ItemId::new("paper_pondshell").unwrap()]
        );
    }

    #[test]
    fn probe_reports_only_catalog_version_and_aggregate_item_count() {
        let fixture = crate::test_support::catalog::fixture_assets();

        assert_eq!(
            probe(&fixture.source, &fixture.cache).unwrap(),
            CatalogProbeReport {
                game_version: "synthetic-1".to_owned(),
                item_count: 1,
            }
        );
    }

    #[test]
    fn fingerprints_an_unversioned_asset_layout_for_explicit_compatibility_approval() {
        let fixture = crate::test_support::catalog::fixture_assets_without_version();
        let catalog = CatalogExtractor::extract(&fixture.source, &fixture.cache).unwrap();
        let item = catalog.item(&ItemId::new("pond_fish").unwrap()).unwrap();

        assert!(catalog
            .version
            .source_game_version
            .starts_with("catalog-sha256:"));
        assert_eq!(item.locations, vec![LocationTag::Pond]);
        assert_eq!(item.text("fra").name, "Poisson d'étang");
    }
}
