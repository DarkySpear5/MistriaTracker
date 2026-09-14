use super::{catalog::JournalCatalog, evidence::JournalEvidence};
use crate::{
    domain::{ItemId, Language, SpoilerMode},
    tracking::ProfileProgress,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub fn snapshot(
    catalog: &JournalCatalog,
    evidence: &JournalEvidence,
    progress: &ProfileProgress,
    language: Language,
    mode: SpoilerMode,
) -> Value {
    let french = language == Language::Fra;
    let all = mode == SpoilerMode::All;
    let mut met = evidence.met.clone();
    let mut gifts = evidence.gifts.clone();
    let mut gift_items = BTreeSet::new();
    for (npc, item, _) in progress.gift_pairs() {
        met.insert(npc.into());
        gifts.entry(npc.into()).or_default().insert(item.into());
        gift_items.insert(item.to_owned());
    }
    let donated: BTreeSet<_> = evidence
        .donated
        .iter()
        .cloned()
        .chain(progress.donated_items().cloned())
        .collect();
    let mut entries = Vec::new();
    let mut keys = BTreeMap::new();
    for (index, (id, definition)) in catalog.entries.iter().enumerate() {
        let found = if let Some(id) = id.strip_prefix("recipe:") {
            evidence.recipes.contains(id)
        } else if let Some(id) = id.strip_prefix("perks:") {
            evidence.perks.contains(id)
        } else if let Some(id) = id.strip_prefix("scrolls:") {
            evidence.spells.contains(id)
        } else if let Some(id) = id.strip_prefix("invitations:") {
            evidence.invitations.contains(id)
        } else {
            donated.contains(id)
                || gift_items.contains(id)
                || ItemId::new(id).is_ok_and(|item| progress.item_is_discovered(&item))
        };
        let key = format!("e{index}");
        keys.insert(id.clone(), key.clone());
        let revealed = all || found;
        entries.push(json!({"key":key,"id":revealed.then_some(id),"category":definition.category,"name":revealed.then_some(if french {&definition.french_name} else {&definition.name}),"description":revealed.then_some(if french {&definition.french_description} else {&definition.description}),"art":key,"found":found,"revealed":revealed,"donated":donated.contains(id),"seasons":if revealed {definition.seasons.clone()} else {vec![]},"places":if revealed {definition.places.clone()} else {vec![]},"hint":hint_for(definition),"sources":[],"ingredients":[]}));
    }
    let revealed_ids: BTreeSet<String> = entries
        .iter()
        .filter_map(|entry| entry["id"].as_str().map(str::to_owned))
        .collect();
    let mut sources: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut sets = Vec::new();
    for set in &catalog.sets {
        let slots: Vec<_> = set.items.iter().filter_map(|id| keys.get(id)).collect();
        for id in &set.items {
            if revealed_ids.contains(id) {
                sources.entry(id.clone()).or_default().push(json!({"view":"museum","key":set.id,"label":if french {&set.french_name} else {&set.name}}));
            }
        }
        sets.push(json!({"id":set.id,"wing":set.wing,"name":if french {&set.french_name} else {&set.name},"items":slots,"completed":set.items.iter().filter(|item|donated.contains(*item)).count(),"total":set.items.len()}));
    }
    let mut villagers = Vec::new();
    for (index, villager) in catalog.villagers.iter().enumerate() {
        let known = met.contains(&villager.id);
        let revealed = all || known;
        let mut groups = BTreeMap::new();
        for (group, ids) in [("loved", &villager.loved), ("liked", &villager.liked)] {
            let mut slots = Vec::new();
            for (slot_index, id) in ids.iter().enumerate() {
                let Some(definition) = catalog.entries.get(id) else {
                    continue;
                };
                let found = gifts
                    .get(&villager.id)
                    .is_some_and(|values| values.contains(id));
                let visible = revealed && (all || found);
                let key = format!("g{index}:{group}:{slot_index}");
                slots.push(json!({"key":key,"entry":visible.then(||keys.get(id)).flatten(),"name":visible.then_some(if french {&definition.french_name} else {&definition.name}),"art":key,"found":found,"revealed":visible}));
                if visible && revealed_ids.contains(id) {
                    sources.entry(id.clone()).or_default().push(json!({"view":"villagers","key":format!("n{index}"),"label":format!("{} · {}", if french {&villager.french_name} else {&villager.name}, if french {if group == "loved" {"Adoré"} else {"Apprécié"}} else if group == "loved" {"Loved"} else {"Liked"})}));
                }
            }
            groups.insert(group, slots);
        }
        villagers.push(json!({"key":format!("n{index}"),"name":revealed.then_some(if french {&villager.french_name} else {&villager.name}),"bio":revealed.then_some(if french {&villager.french_bio} else {&villager.bio}),"art":format!("n{index}"),"met":known,"revealed":revealed,"loved":if revealed {groups.remove("loved").unwrap_or_default()} else {vec![]},"liked":if revealed {groups.remove("liked").unwrap_or_default()} else {vec![]}}));
    }
    for entry in &mut entries {
        let Some(id) = entry["id"].as_str().map(str::to_owned) else {
            continue;
        };
        let definition = &catalog.entries[&id];
        let mut links = sources.remove(&id).unwrap_or_default();
        links.insert(
            0,
            json!({"view":"encyclopedia","key":definition.category,"label":definition.category}),
        );
        if let Some(key) = &definition.recipe_key {
            let recipe = format!("recipe:{key}");
            if id != recipe && revealed_ids.contains(&recipe) {
                links.push(json!({"view":"encyclopedia","key":"recipes","entry":keys.get(&recipe),"label":if french {"Recette"} else {"Recipe"}}));
            }
        }
        if let Some(item) = &definition.related_item {
            if revealed_ids.contains(item) {
                links.push(json!({"view":"encyclopedia","key":catalog.entries[item].category,"entry":keys.get(item),"label":if french {"Objet fabriqué"} else {"Crafted item"}}));
            }
        }
        entry["sources"] = json!(links);
        entry["ingredients"] = json!(definition.ingredients.iter().map(|(id,count)| json!({"key":revealed_ids.contains(id).then(||keys.get(id)).flatten(),"name":revealed_ids.contains(id).then(||catalog.entries.get(id).map(|item|if french {&item.french_name} else {&item.name})).flatten(),"count":count})).collect::<Vec<_>>());
    }
    let mut categories = BTreeMap::<String, (usize, usize)>::new();
    for entry in &entries {
        let count = categories
            .entry(entry["category"].as_str().unwrap().into())
            .or_default();
        count.1 += 1;
        count.0 += usize::from(entry["found"].as_bool().unwrap_or(false));
    }
    json!({"entries":entries,"sets":sets,"villagers":villagers,"categories":categories.into_iter().map(|(id,(completed,total))|json!({"id":id,"completed":completed,"total":total})).collect::<Vec<_>>(),"profile":{"name":evidence.name,"farm":evidence.farm},"stats":evidence.stats})
}

/// Art tokens are resolved against the current spoiler-filtered snapshot, never a raw path.
pub fn artwork_for(catalog: &JournalCatalog, snapshot: &Value, key: &str) -> Option<String> {
    for (index, entry) in snapshot["entries"].as_array()?.iter().enumerate() {
        if entry["art"].as_str() == Some(key) {
            return catalog.entries.values().nth(index).map(|e| {
                protected_sprite(&e.sprite, entry["revealed"].as_bool().unwrap_or(false))
            });
        }
    }
    for (index, villager) in snapshot["villagers"].as_array()?.iter().enumerate() {
        if villager["art"].as_str() == Some(key) {
            return Some(protected_sprite(
                &catalog.villagers[index].portrait,
                villager["revealed"].as_bool().unwrap_or(false),
            ));
        }
        for group in ["loved", "liked"] {
            for gift in villager[group].as_array()? {
                if gift["art"].as_str() == Some(key) {
                    let slot: usize = key.rsplit(':').next()?.parse().ok()?;
                    let ids = if group == "loved" {
                        &catalog.villagers[index].loved
                    } else {
                        &catalog.villagers[index].liked
                    };
                    return catalog.entries.get(ids.get(slot)?).map(|e| {
                        protected_sprite(&e.sprite, gift["revealed"].as_bool().unwrap_or(false))
                    });
                }
            }
        }
    }
    None
}

fn protected_sprite(sprite: &str, revealed: bool) -> String {
    if revealed {
        sprite.to_owned()
    } else {
        format!("{sprite}_hidden")
    }
}

/// A hint is intentionally a broad, non-identifying condition. Exact locations,
/// names, and seasons stay behind the spoiler boundary.
fn hint_for(entry: &crate::journal::catalog::Entry) -> &'static str {
    let places = entry
        .places
        .iter()
        .map(|place| place.to_ascii_lowercase())
        .collect::<Vec<_>>();
    if places.iter().any(|place| place.contains("east")) {
        return "region_east";
    }
    if entry.category == "fish" {
        if places.iter().any(|place| place.contains("mine")) {
            return "fish_cave";
        }
        if places
            .iter()
            .any(|place| place.contains("ocean") || place.contains("coast"))
        {
            return "fish_coast";
        }
        if places.iter().any(|place| place.contains("river")) {
            return "fish_river";
        }
        if places.iter().any(|place| place.contains("pond")) {
            return "fish_pond";
        }
    }
    match entry.category.as_str() {
        "crops" => "crops",
        "bugs" => "bugs",
        "artifacts" => "artifacts",
        "recipes" => "recipes",
        _ => "generic",
    }
}
