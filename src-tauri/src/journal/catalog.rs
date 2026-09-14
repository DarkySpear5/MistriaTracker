use crate::catalog::{AssetsZip, CatalogError};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, io::Read};
use toml::Value;
use zip::ZipArchive;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub category: String,
    pub name: String,
    pub french_name: String,
    pub description: String,
    pub french_description: String,
    pub sprite: String,
    pub seasons: Vec<String>,
    pub places: Vec<String>,
    pub recipe_key: Option<String>,
    pub ingredients: Vec<(String, u64)>,
    pub related_item: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MuseumSet {
    pub id: String,
    pub wing: String,
    pub name: String,
    pub french_name: String,
    pub items: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Villager {
    pub id: String,
    pub name: String,
    pub french_name: String,
    pub bio: String,
    pub french_bio: String,
    pub portrait: String,
    pub loved: Vec<String>,
    pub liked: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct JournalCatalog {
    pub entries: BTreeMap<String, Entry>,
    pub sets: Vec<MuseumSet>,
    pub villagers: Vec<Villager>,
    pub artwork: BTreeMap<String, String>,
}

pub fn string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}
fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
fn translated(translations: &Value, path: &str, id: &str, field: &str, fallback: &str) -> String {
    let base = path
        .trim_start_matches("assets/fiddle/")
        .trim_end_matches(".toml");
    let key = if id.is_empty() {
        format!("{base}/{field}")
    } else {
        format!("{base}/{id}/{field}")
    };
    translations
        .get("asset_properties")
        .and_then(|v| v.get(&key))
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_owned()
}

#[cfg(test)]
mod localization_tests {
    use super::*;
    #[test]
    fn game_translation_keys_omit_the_assets_fiddle_prefix() {
        let translations: Value = toml::from_str("[asset_properties]\n\"items/fish/river/trout/name\" = \"Truite\"\n\"museum_wings/fish/sets/spring/name\" = \"Poissons du printemps\"").unwrap();
        assert_eq!(
            translated(
                &translations,
                "assets/fiddle/items/fish/river.toml",
                "trout",
                "name",
                "Trout"
            ),
            "Truite"
        );
        assert_eq!(
            translated(
                &translations,
                "assets/fiddle/museum_wings/fish.toml",
                "sets/spring",
                "name",
                "Spring Fish"
            ),
            "Poissons du printemps"
        );
    }
}
fn category(path: &str, item: &Value) -> &'static str {
    let tags = strings(item.get("tags"));
    if path.contains("/fish/") {
        "fish"
    } else if path.contains("/furniture/") {
        "furniture"
    } else if tags.iter().any(|tag| tag == "crop") {
        "crops"
    } else if tags.iter().any(|tag| tag == "forageable") {
        "forageables"
    } else if path.ends_with("/bugs.toml") {
        "bugs"
    } else if path.ends_with("/artifacts.toml") {
        "artifacts"
    } else if path.ends_with("/cooked_dishes.toml") {
        "dishes"
    } else if path.ends_with("/ranching.toml") || path.ends_with("/mill.toml") {
        "ranching"
    } else if path.ends_with("/song_crystals.toml") {
        "songs"
    } else if path.ends_with("/dating.toml") || path.ends_with("/10_heart_special_items.toml") {
        "dating"
    } else if path.ends_with("/blacksmithing.toml") || path.ends_with("/armor.toml") {
        "blacksmithing"
    } else if path.ends_with("/materials.toml") || path.ends_with("/monster_drops.toml") {
        "materials"
    } else {
        "other"
    }
}

impl JournalCatalog {
    pub fn extract(source: &AssetsZip) -> Result<Self, CatalogError> {
        let mut archive = ZipArchive::new(File::open(source.as_path())?)?;
        let mut documents = BTreeMap::new();
        let mut artwork = BTreeMap::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            let path = entry.name().to_owned();
            if entry.enclosed_name().is_none() {
                return Err(CatalogError::InvalidSource("unsafe journal source path"));
            }
            if path.starts_with("assets/animations/") && path.ends_with(".png") {
                let sprite = path
                    .rsplit('/')
                    .next()
                    .unwrap()
                    .trim_end_matches(".png")
                    .to_owned();
                artwork.entry(sprite).or_insert(path.clone());
            }
            let allowed = path.starts_with("assets/fiddle/items/")
                || path.starts_with("assets/fiddle/museum_wings/")
                || path.starts_with("assets/fiddle/npcs/")
                || path.starts_with("assets/fiddle/ui/skill_menu/")
                || matches!(
                    path.as_str(),
                    "assets/fiddle/fish.toml"
                        | "assets/fiddle/bugs.toml"
                        | "assets/fiddle/perks.toml"
                        | "assets/fiddle/spells.toml"
                        | "assets/fiddle/dates.toml"
                        | "assets/localization/translations/fra.meta.toml"
                );
            if !allowed || !path.ends_with(".toml") {
                continue;
            }
            if entry.size() > 16 * 1024 * 1024 {
                return Err(CatalogError::InvalidSource("journal definition too large"));
            }
            let mut text = String::new();
            entry.read_to_string(&mut text)?;
            documents.insert(path, toml::from_str::<Value>(&text)?);
        }
        let translations = documents
            .get("assets/localization/translations/fra.meta.toml")
            .cloned()
            .unwrap_or(Value::Table(Default::default()));
        let mut catalog = Self {
            artwork,
            ..Self::default()
        };
        for (path, document) in &documents {
            if !path.starts_with("assets/fiddle/items/") {
                continue;
            }
            let tables = document
                .get("items")
                .unwrap_or(document)
                .as_table()
                .ok_or(CatalogError::InvalidSource("invalid item table"))?;
            for (id, value) in tables {
                let name = string(value, "name");
                if name.is_empty() {
                    continue;
                }
                let description = string(value, "description");
                let mut item = Entry {
                    id: id.clone(),
                    category: category(path, value).into(),
                    french_name: translated(&translations, path, id, "name", &name),
                    name,
                    french_description: translated(
                        &translations,
                        path,
                        id,
                        "description",
                        &description,
                    ),
                    description,
                    sprite: string(value, "icon_sprite"),
                    seasons: strings(value.get("seasons")),
                    places: strings(value.get("locations")),
                    ..Entry::default()
                };
                if item.sprite.is_empty() {
                    item.sprite = format!("spr_ui_item_{id}");
                }
                if item.category == "fish" {
                    for place in ["river", "pond", "ocean"] {
                        if path.ends_with(&format!("/{place}.toml")) {
                            item.places.push(place.into());
                        }
                    }
                }
                if path.contains("/mines/") {
                    item.places.push("mines".into());
                }
                if let Some(recipe) = value.get("recipe").and_then(Value::as_array) {
                    let recipe_key = string(value, "recipe_key");
                    let recipe_key = if recipe_key.is_empty() {
                        id.clone()
                    } else {
                        recipe_key
                    };
                    item.recipe_key = Some(recipe_key.clone());
                    let mut learned = item.clone();
                    learned.id = format!("recipe:{recipe_key}");
                    learned.category = "recipes".into();
                    learned.related_item = Some(id.clone());
                    learned.ingredients = recipe
                        .iter()
                        .filter_map(|ingredient| {
                            let item = string(ingredient, "item");
                            let count = ingredient.get("count").and_then(Value::as_integer)?;
                            (!item.is_empty() && count > 0).then_some((item, count as u64))
                        })
                        .collect();
                    catalog.entries.entry(learned.id.clone()).or_insert(learned);
                }
                catalog.entries.insert(id.clone(), item);
            }
        }
        for (path, document) in &documents {
            if path.starts_with("assets/fiddle/museum_wings/") {
                let wing = path.rsplit('/').next().unwrap().trim_end_matches(".toml");
                if let Some(sets) = document.get("sets").and_then(Value::as_table) {
                    for (id, set) in sets {
                        let name = string(set, "name");
                        let items = strings(set.get("items"));
                        let french_name =
                            translated(&translations, path, &format!("sets/{id}"), "name", &name);
                        catalog.sets.push(MuseumSet {
                            id: format!("{wing}:{id}"),
                            wing: wing.into(),
                            name,
                            french_name,
                            items: items.clone(),
                        });
                        for item in items {
                            if let Some(entry) = catalog.entries.get_mut(&item) {
                                for season in ["spring", "summer", "fall", "winter"] {
                                    if id.split('_').any(|part| part == season)
                                        && !entry.seasons.contains(&season.to_owned())
                                    {
                                        entry.seasons.push(season.into());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if path.starts_with("assets/fiddle/npcs/") {
                let id = path
                    .rsplit('/')
                    .next()
                    .unwrap()
                    .trim_end_matches(".toml")
                    .to_owned();
                let name = string(document, "name");
                if name.is_empty() {
                    continue;
                }
                let bio = string(document, "bio");
                let portrait = [
                    format!("spr_portrait_{id}_spring_neutral"),
                    format!("spr_portrait_{id}_neutral"),
                    string(document, "icon_sprite"),
                ]
                .into_iter()
                .find(|sprite| catalog.artwork.contains_key(sprite))
                .unwrap_or_default();
                catalog.villagers.push(Villager {
                    french_name: translated(&translations, path, "", "name", &name),
                    name,
                    french_bio: translated(&translations, path, "", "bio", &bio),
                    bio,
                    id,
                    portrait,
                    loved: strings(document.get("loved_gifts")),
                    liked: strings(document.get("liked_gifts")),
                });
            }
        }
        for (path, category) in [
            ("assets/fiddle/perks.toml", "perks"),
            ("assets/fiddle/spells.toml", "scrolls"),
            ("assets/fiddle/dates.toml", "invitations"),
        ] {
            if let Some(table) = documents.get(path).and_then(Value::as_table) {
                for (id, value) in table {
                    if id == "default" {
                        continue;
                    }
                    let name = string(value, "name");
                    if name.is_empty() || name == "MISSING" {
                        continue;
                    }
                    let description = string(value, "description");
                    catalog.entries.insert(
                        format!("{category}:{id}"),
                        Entry {
                            id: format!("{category}:{id}"),
                            category: category.into(),
                            french_name: translated(&translations, path, id, "name", &name),
                            name,
                            french_description: translated(
                                &translations,
                                path,
                                id,
                                "description",
                                &description,
                            ),
                            description,
                            sprite: if category == "scrolls" {
                                string(value, "icon_key")
                            } else {
                                string(value, "icon")
                            },
                            ..Entry::default()
                        },
                    );
                }
            }
        }
        for (path, document) in &documents {
            if path.starts_with("assets/fiddle/ui/skill_menu/") {
                if let Some(table) = document.as_table() {
                    for values in table.values().filter_map(Value::as_array) {
                        for perk in values {
                            if let Some(entry) = catalog
                                .entries
                                .get_mut(&format!("perks:{}", string(perk, "perk")))
                            {
                                entry.sprite = string(perk, "icon");
                            }
                        }
                    }
                }
            }
        }
        for path in ["assets/fiddle/fish.toml", "assets/fiddle/bugs.toml"] {
            if let Some(table) = documents.get(path).and_then(Value::as_table) {
                for (id, value) in table {
                    if id == "default" {
                        continue;
                    }
                    let item_id = string(value, "item");
                    let item_id = if item_id.is_empty() || item_id == "<..>" {
                        id.as_str()
                    } else {
                        &item_id
                    };
                    if let Some(entry) = catalog.entries.get_mut(item_id) {
                        let default = table.get("default");
                        let seasons = value
                            .get("seasons")
                            .or_else(|| default.and_then(|d| d.get("seasons")));
                        entry.seasons = if seasons == Some(&Value::Boolean(false)) {
                            ["spring", "summer", "fall", "winter"]
                                .map(str::to_owned)
                                .to_vec()
                        } else {
                            strings(seasons)
                        };
                        if let Some(water) = value
                            .get("water_type")
                            .or_else(|| default.and_then(|d| d.get("water_type")))
                            .and_then(Value::as_str)
                        {
                            if !entry.places.contains(&water.to_owned()) {
                                entry.places.push(water.into());
                            }
                        }
                    }
                }
            }
        }
        Ok(catalog)
    }
}
