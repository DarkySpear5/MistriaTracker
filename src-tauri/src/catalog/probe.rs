use super::{AssetsZip, CatalogCacheDir, CatalogError, CatalogExtractor};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CatalogProbeReport {
    pub game_version: String,
    pub item_count: usize,
}

/// Reads an asset archive into memory and reports only aggregate compatibility evidence.
pub fn probe(
    source: &AssetsZip,
    cache: &CatalogCacheDir,
) -> Result<CatalogProbeReport, CatalogError> {
    let catalog = CatalogExtractor::extract(source, cache)?;
    let item_count = catalog.items().count();
    Ok(CatalogProbeReport {
        game_version: catalog.version.source_game_version,
        item_count,
    })
}
