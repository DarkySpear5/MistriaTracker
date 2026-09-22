use super::{availability::LocationTag, localization::LocalizedItemText};
use crate::domain::ItemId;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogVersion {
    pub source_game_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogItem {
    pub id: ItemId,
    pub icon_sprite: Option<String>,
    pub seasons: Vec<String>,
    pub locations: Vec<LocationTag>,
    text: BTreeMap<String, LocalizedItemText>,
}

impl CatalogItem {
    pub(crate) fn new(
        id: ItemId,
        icon_sprite: Option<String>,
        seasons: Vec<String>,
        locations: Vec<LocationTag>,
        text: BTreeMap<String, LocalizedItemText>,
    ) -> Self {
        Self {
            id,
            icon_sprite,
            seasons,
            locations,
            text,
        }
    }

    pub fn text(&self, language: &str) -> &LocalizedItemText {
        self.text.get(language).unwrap_or_else(|| {
            self.text
                .get("eng")
                .expect("English catalog text is required")
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub version: CatalogVersion,
    items: BTreeMap<String, CatalogItem>,
}

impl Catalog {
    pub(crate) fn new(version: CatalogVersion, items: Vec<CatalogItem>) -> Self {
        let items = items
            .into_iter()
            .map(|item| (item.id.as_str().to_owned(), item))
            .collect();
        Self { version, items }
    }

    pub fn item(&self, id: &ItemId) -> Option<&CatalogItem> {
        self.items.get(id.as_str())
    }

    pub fn item_ids(&self) -> Vec<ItemId> {
        self.items.values().map(|item| item.id.clone()).collect()
    }

    pub fn items(&self) -> impl Iterator<Item = &CatalogItem> {
        self.items.values()
    }
}
