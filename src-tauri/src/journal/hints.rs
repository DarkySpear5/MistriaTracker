use super::catalog::{Entry, JournalCatalog};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use toml::Value;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HintFacts {
    pub areas: Vec<String>,
    pub seasons: Vec<String>,
    pub recipe_source: Option<String>,
}

#[derive(Serialize)]
pub struct Hint {
    kind: &'static str,
    activity: Option<&'static str>,
    areas: Vec<String>,
    seasons: Vec<String>,
    source: Option<String>,
}

fn add(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_owned());
    }
}

fn season(value: &str) -> Option<&'static str> {
    match value {
        "spring" => Some("spring"),
        "summer" => Some("summer"),
        "fall" => Some("fall"),
        "winter" => Some("winter"),
        _ => None,
    }
}

fn area(value: &str) -> Option<&'static str> {
    match value {
        "pond" => Some("pond"),
        "river" => Some("river"),
        "ocean" => Some("ocean"),
        "mines" | "dungeon" => Some("mines"),
        "beach" => Some("beach"),
        "deep_woods" => Some("deep_woods"),
        "standard" | "outdoors" => Some("outdoors"),
        "narrows" => Some("narrows"),
        "eastern_road" => Some("eastern_road"),
        "haydens_farm" => Some("haydens_farm"),
        "western_ruins" => Some("western_ruins"),
        "farm" => Some("farm"),
        _ => None,
    }
}

fn source_priority(value: &str) -> u8 {
    match value {
        "store" => 6,
        "mail" => 5,
        "quest" => 4,
        "museum" => 3,
        "random" => 2,
        "start" => 1,
        _ => 0,
    }
}

impl HintFacts {
    fn add_area(&mut self, value: &str) {
        if let Some(value) = area(value) {
            add(&mut self.areas, value);
        }
    }

    fn add_seasons(&mut self, value: Option<&Value>, false_means_all: bool) {
        if false_means_all && value == Some(&Value::Boolean(false)) {
            for season in ["spring", "summer", "fall", "winter"] {
                add(&mut self.seasons, season);
            }
        } else {
            for value in strings(value) {
                if let Some(season) = season(value) {
                    add(&mut self.seasons, season);
                }
            }
        }
    }

    pub fn add_source(&mut self, source: &str) {
        if self
            .recipe_source
            .as_deref()
            .is_none_or(|current| source_priority(source) > source_priority(current))
        {
            if source_priority(source) > 0 {
                self.recipe_source = Some(source.to_owned());
            }
        }
    }
}

/// Hint payloads contain only finite codes, never hidden names or IDs.
pub fn hint_for(entry: &Entry, gift: bool) -> Hint {
    let activity = match entry.category.as_str() {
        "fish" => Some("fish"),
        "bugs" => Some("bugs"),
        "crops" => Some("crops"),
        "forageables" => Some("forageables"),
        "artifacts" => Some("artifacts"),
        "dishes" => Some("dishes"),
        "recipes" => Some("recipes"),
        "furniture" => Some("furniture"),
        "materials" => Some("materials"),
        "ranching" => Some("ranching"),
        "blacksmithing" => Some("blacksmithing"),
        _ => None,
    };
    let kind = if gift {
        "gift"
    } else {
        match entry.category.as_str() {
            "fish" => "fish",
            "bugs" => "bugs",
            "crops" => "crops",
            "forageables" => "forageables",
            "artifacts" => "artifacts",
            "recipes" => "recipes",
            _ => "generic",
        }
    };
    Hint {
        kind,
        activity: gift.then_some(activity).flatten(),
        areas: entry.hint_facts.areas.clone(),
        seasons: entry.hint_facts.seasons.clone(),
        source: (kind == "recipes")
            .then_some(entry.hint_facts.recipe_source.clone())
            .flatten(),
    }
}

fn strings(value: Option<&Value>) -> Vec<&str> {
    match value {
        Some(Value::String(value)) => vec![value.as_str()],
        Some(Value::Array(values)) => values.iter().filter_map(Value::as_str).collect(),
        _ => vec![],
    }
}

fn field<'a>(entry: &'a Value, default: Option<&'a Value>, key: &str) -> Option<&'a Value> {
    entry
        .get(key)
        .or_else(|| default.and_then(|value| value.get(key)))
}

fn apply_fish(catalog: &mut JournalCatalog, document: &Value) {
    let Some(table) = document.as_table() else {
        return;
    };
    let default = table.get("default");
    for (id, definition) in table {
        if id == "default" {
            continue;
        }
        let item_id = definition
            .get("item")
            .and_then(Value::as_str)
            .filter(|id| *id != "<..>")
            .unwrap_or(id);
        let Some(item) = catalog.entries.get_mut(item_id) else {
            continue;
        };
        if item.category != "fish" {
            continue;
        }
        item.hint_facts
            .add_seasons(field(definition, default, "seasons"), true);
        let retrieval = strings(field(definition, default, "retrieval"));
        let mined = retrieval.contains(&"mines");
        if mined {
            item.hint_facts.add_area("mines");
        }
        for location in strings(field(definition, default, "locations")) {
            if location == "deep_woods" {
                item.hint_facts.add_area(location);
            }
        }
        // A mine-only fish inherits default river water, but never spawns there.
        if !mined
            || retrieval
                .iter()
                .any(|value| matches!(*value, "fishing" | "divespot"))
        {
            let water = if mined {
                definition.get("water_type")
            } else {
                field(definition, default, "water_type")
            };
            for water in strings(water) {
                if matches!(water, "pond" | "river" | "ocean") {
                    item.hint_facts.add_area(water);
                }
            }
        }
    }
}

fn apply_bugs(catalog: &mut JournalCatalog, document: &Value) {
    let Some(table) = document.as_table() else {
        return;
    };
    let default = table.get("default");
    for (id, definition) in table {
        if id == "default" {
            continue;
        }
        let Some(item) = catalog.entries.get_mut(id) else {
            continue;
        };
        if item.category != "bugs" {
            continue;
        }
        item.hint_facts
            .add_seasons(field(definition, default, "seasons"), false);
        if definition
            .get("dungeon_biome")
            .and_then(Value::as_str)
            .is_some()
        {
            item.hint_facts.add_area("mines");
        }
        for tag in strings(field(definition, default, "tag")) {
            if matches!(tag, "beach" | "deep_woods" | "mines" | "standard") {
                item.hint_facts.add_area(tag);
            }
        }
    }
}

fn apply_crops(catalog: &mut JournalCatalog, document: &Value) {
    let Some(table) = document.as_table() else {
        return;
    };
    for (id, definition) in table {
        let Some(item) = catalog.entries.get_mut(id) else {
            continue;
        };
        if matches!(item.category.as_str(), "crops" | "forageables") {
            item.hint_facts
                .add_seasons(definition.get("seasons"), false);
        }
    }
}

fn apply_forageables(catalog: &mut JournalCatalog, document: &Value) {
    for season_name in ["spring", "summer", "fall", "winter"] {
        let Some(table) = document.get(season_name).and_then(Value::as_table) else {
            continue;
        };
        for rarity in ["common", "uncommon", "rare", "legendary"] {
            for id in strings(table.get(rarity)) {
                if let Some(item) = catalog.entries.get_mut(id) {
                    if item.category == "forageables" {
                        add(&mut item.hint_facts.seasons, season_name);
                        item.hint_facts.add_area("outdoors");
                    }
                }
            }
        }
    }
    for id in strings(document.get("sand_forageables")) {
        if let Some(item) = catalog.entries.get_mut(id) {
            if item.category == "forageables" {
                item.hint_facts.areas.retain(|area| area != "outdoors");
                item.hint_facts.add_area("beach");
            }
        }
    }
}

fn apply_artifacts(catalog: &mut JournalCatalog, document: &Value) {
    let Some(locations) = document.get("locations").and_then(Value::as_table) else {
        return;
    };
    for (location, group) in locations {
        let (Some(_), Some(group)) = (area(location), group.as_str()) else {
            continue;
        };
        let Some(set) = catalog
            .sets
            .iter()
            .find(|set| set.id == format!("archaeology:{group}"))
        else {
            continue;
        };
        for id in &set.items {
            if let Some(item) = catalog.entries.get_mut(id) {
                if item.category == "artifacts" {
                    item.hint_facts.add_area(location);
                }
            }
        }
    }
}

fn recipe_references(value: &Value, recipes: &mut Vec<String>, bundles: &mut Vec<String>) {
    match value {
        Value::Table(table) => {
            for key in ["recipe_scroll", "crafting_scroll"] {
                if let Some(id) = table.get(key).and_then(Value::as_str) {
                    recipes.push(id.to_owned());
                }
            }
            if table.get("include_recipe").and_then(Value::as_bool) == Some(true) {
                if let Some(id) = table.get("item").and_then(Value::as_str) {
                    bundles.push(id.to_owned());
                }
            }
            for nested in table.values() {
                recipe_references(nested, recipes, bundles);
            }
        }
        Value::Array(values) => {
            for nested in values {
                recipe_references(nested, recipes, bundles);
            }
        }
        _ => {}
    }
}

fn source_for_path(path: &str) -> Option<&'static str> {
    match path {
        "assets/fiddle/stores.toml" | "assets/fiddle/festivals.toml" => Some("store"),
        "assets/fiddle/letters.toml" => Some("mail"),
        "assets/fiddle/quests/fetch_quests.toml"
        | "assets/fiddle/quests/story_quests.toml"
        | "assets/fiddle/quests/stillwell_challenges.toml"
        | "assets/fiddle/quests/tali_challenges.toml"
        | "assets/fiddle/cutscenes.toml" => Some("quest"),
        "assets/fiddle/wishing_well.toml" | "assets/fiddle/chicken_statue.toml" => Some("random"),
        path if path.starts_with("assets/fiddle/museum_wings/") => Some("museum"),
        _ => None,
    }
}

fn apply_recipes(catalog: &mut JournalCatalog, documents: &BTreeMap<String, Value>) {
    for (path, document) in documents {
        let Some(source) = source_for_path(path) else {
            continue;
        };
        let (mut recipes, mut bundles) = (Vec::new(), Vec::new());
        recipe_references(document, &mut recipes, &mut bundles);
        for item in bundles.drain(..) {
            if let Some(key) = catalog
                .entries
                .get(&item)
                .and_then(|entry| entry.recipe_key.clone())
            {
                recipes.push(key);
            }
        }
        for recipe in recipes {
            if let Some(entry) = catalog.entries.get_mut(&format!("recipe:{recipe}")) {
                entry.hint_facts.add_source(source);
            }
        }
    }
}

pub fn populate_hint_facts(catalog: &mut JournalCatalog, documents: &BTreeMap<String, Value>) {
    for (path, apply) in [
        (
            "assets/fiddle/fish.toml",
            apply_fish as fn(&mut JournalCatalog, &Value),
        ),
        ("assets/fiddle/bugs.toml", apply_bugs),
        ("assets/fiddle/object_prototypes/crop.toml", apply_crops),
        ("assets/fiddle/forageables.toml", apply_forageables),
        ("assets/fiddle/artifacts.toml", apply_artifacts),
    ] {
        if let Some(document) = documents.get(path) {
            apply(catalog, document);
        }
    }
    apply_recipes(catalog, documents);
}
