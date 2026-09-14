use crate::save::{evidence::EvidenceError, vault::Vault};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JournalEvidence {
    pub name: String,
    pub farm: String,
    pub donated: BTreeSet<String>,
    pub recipes: BTreeSet<String>,
    pub met: BTreeSet<String>,
    pub gifts: BTreeMap<String, BTreeSet<String>>,
    pub perks: BTreeSet<String>,
    pub spells: BTreeSet<String>,
    pub invitations: BTreeSet<String>,
    pub stats: BTreeMap<String, u64>,
}
fn string_set(value: Option<&Value>, field: &str) -> Result<BTreeSet<String>, EvidenceError> {
    let Some(value) = value else {
        return Ok(BTreeSet::new());
    };
    let Some(array) = value.as_array() else {
        return Err(EvidenceError::InvalidShape(field.into()));
    };
    array
        .iter()
        .map(|v| {
            v.as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| EvidenceError::InvalidShape(field.into()))
        })
        .collect()
}
impl JournalEvidence {
    /// Called only after the existing backup parser verifies game version 1.0.4.
    pub fn from_vault(vault: &Vault) -> Result<Self, EvidenceError> {
        let player = vault
            .section("player")
            .ok_or(EvidenceError::MissingSection("player"))?;
        let header = vault
            .section("header")
            .ok_or(EvidenceError::MissingSection("header"))?;
        let mut evidence = Self {
            name: header
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .into(),
            farm: header
                .get("farm_name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .into(),
            recipes: string_set(player.get("recipe_unlocks"), "player.recipe_unlocks")?,
            perks: string_set(player.get("perks"), "player.perks")?,
            spells: string_set(player.get("spells_learned"), "player.spells_learned")?,
            invitations: string_set(player.get("date_unlocks"), "player.date_unlocks")?,
            donated: string_set(
                vault
                    .section("gamedata")
                    .and_then(|v| v.get("museum_progress")),
                "gamedata.museum_progress",
            )?,
            ..Self::default()
        };
        if let Some(npcs) = vault.section("npcs").and_then(Value::as_object) {
            for (npc, details) in npcs {
                let mut gifts = string_set(details.get("gifts_given"), "npcs.gifts_given")?;
                gifts.extend(string_set(
                    details.get("known_gift_preferences"),
                    "npcs.known_gift_preferences",
                )?);
                if !gifts.is_empty() {
                    evidence.met.insert(npc.clone());
                }
                evidence.gifts.insert(npc.clone(), gifts);
            }
        }
        if let Some(stats) = vault.section("game_stats") {
            if let Some(spoken) = stats.get("npcs_spoken_to").and_then(Value::as_object) {
                for (npc, count) in spoken {
                    if count.as_u64().unwrap_or_default() > 0 {
                        evidence.met.insert(npc.clone());
                    }
                }
            }
            for field in [
                "fish_caught",
                "bugs_caught",
                "crop_harvests",
                "forageable_harvests",
                "gifts_given",
                "items_cooked",
            ] {
                if let Some(values) = stats.get(field).and_then(Value::as_array) {
                    evidence.stats.insert(field.into(), values.len() as u64);
                }
            }
        }
        Ok(evidence)
    }
}
