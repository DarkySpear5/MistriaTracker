use crate::{
    compatibility::{CompatibilityMatrix, SaveParserDecision},
    domain::{GiftReaction, ItemId, NpcId, ProfileId},
    save::vault::Vault,
};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub(crate) const SAVE_PARSER_VERSION: u16 = 1;

#[derive(Clone, Debug, Serialize)]
pub struct ImportedEvidence {
    pub profile_id: ProfileId,
    #[serde(flatten)]
    pub fact: EvidenceFact,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EvidenceFact {
    ProfileSeen {
        name: String,
        farm_name: String,
    },
    ItemOwned {
        item_id: ItemId,
        count: u64,
    },
    NpcMet {
        npc_id: NpcId,
    },
    GiftKnown {
        npc_id: NpcId,
        item_id: ItemId,
        reaction: Option<GiftReaction>,
    },
    MuseumDonated {
        item_id: ItemId,
        set_id: String,
    },
    CollectionKnown {
        collection: String,
        count: u64,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
    #[error("vault is missing required section {0}")]
    MissingSection(&'static str),
    #[error("vault uses an unsupported game version {0}")]
    UnsupportedVersion(String),
    #[error("vault version {0} is pending a verified parser probe")]
    UnverifiedVersion(String),
    #[error("vault has an invalid verified field shape: {0}")]
    InvalidShape(String),
}

pub struct EvidenceExtractor<'a> {
    profile_id: ProfileId,
    compatibility: &'a CompatibilityMatrix,
}

impl<'a> EvidenceExtractor<'a> {
    pub fn new(profile_id: ProfileId, compatibility: &'a CompatibilityMatrix) -> Self {
        Self {
            profile_id,
            compatibility,
        }
    }

    pub fn extract(&self, vault: &Vault) -> Result<Vec<ImportedEvidence>, EvidenceError> {
        let version = Self::game_version(vault)?;
        match self
            .compatibility
            .save_parser_decision(&version, SAVE_PARSER_VERSION)
        {
            SaveParserDecision::Verified => {}
            SaveParserDecision::ProbeRequired => {
                return Err(EvidenceError::UnverifiedVersion(version))
            }
            SaveParserDecision::Unsupported => {
                return Err(EvidenceError::UnsupportedVersion(version))
            }
        }

        // 1.0.4 and 1.0.5 were verified against user-approved snapshots. Their approved
        // catch-up surface is deliberately narrow: the profile header and the
        // stable `items_acquired` list. Other save fields are not inferred.
        if matches!(version.as_str(), "1.0.4" | "1.0.5") {
            return self.extract_v1_0_4(vault);
        }

        let header = required_object(vault, "header")?;
        let mut evidence = vec![self.fact(EvidenceFact::ProfileSeen {
            name: required_string(header, "name", "header.name")?.to_owned(),
            farm_name: required_string(header, "farm_name", "header.farm_name")?.to_owned(),
        })];

        if let Some(player) = optional_object(vault, "player")? {
            self.extract_player(player, &mut evidence)?;
        }
        if let Some(npcs) = optional_object(vault, "npcs")? {
            self.extract_npcs(npcs, &mut evidence)?;
        }
        if let Some(gamedata) = optional_object(vault, "gamedata")? {
            self.extract_museum(gamedata, &mut evidence)?;
        }
        if let Some(stats) = optional_object(vault, "game_stats")? {
            self.extract_collections(stats, &mut evidence)?;
        }
        Ok(evidence)
    }

    pub fn game_version(vault: &Vault) -> Result<String, EvidenceError> {
        game_version(required_object(vault, "info")?)
    }

    fn extract_v1_0_4(&self, vault: &Vault) -> Result<Vec<ImportedEvidence>, EvidenceError> {
        let header = required_object(vault, "header")?;
        let mut evidence = vec![self.fact(EvidenceFact::ProfileSeen {
            name: required_string(header, "name", "header.name")?.to_owned(),
            farm_name: required_string(header, "farm_name", "header.farm_name")?.to_owned(),
        })];
        let player = required_object(vault, "player")?;
        let items = required_value(player, "items_acquired", "player.items_acquired")?
            .as_array()
            .ok_or_else(|| invalid("player.items_acquired"))?;
        let mut seen = BTreeSet::new();
        for item in items {
            let item = item
                .as_str()
                .ok_or_else(|| invalid("player.items_acquired entry"))?;
            if seen.insert(item.to_owned()) {
                evidence.push(self.fact(EvidenceFact::ItemOwned {
                    item_id: item_id(item)?,
                    count: 1,
                }));
            }
        }
        Ok(evidence)
    }

    fn extract_player(
        &self,
        player: &Map<String, Value>,
        evidence: &mut Vec<ImportedEvidence>,
    ) -> Result<(), EvidenceError> {
        if let Some(items) = player.get("items_acquired") {
            match items {
                Value::Object(items) => {
                    for (item, count) in items {
                        let count = unsigned(count, "player.items_acquired count")?;
                        if count > 0 {
                            evidence.push(self.fact(EvidenceFact::ItemOwned {
                                item_id: item_id(item)?,
                                count,
                            }));
                        }
                    }
                }
                Value::Array(items) => {
                    let mut seen = BTreeSet::new();
                    for item in items {
                        let item = item
                            .as_str()
                            .ok_or_else(|| invalid("player.items_acquired entry"))?;
                        if seen.insert(item.to_owned()) {
                            let item_id = item_id(item)?;
                            evidence.push(self.fact(EvidenceFact::ItemOwned { item_id, count: 1 }));
                        }
                    }
                }
                _ => return Err(invalid("player.items_acquired")),
            }
        }
        if let Some(inventory) = player.get("inventory") {
            let inventory = inventory
                .as_array()
                .ok_or_else(|| invalid("player.inventory"))?;
            for entry in inventory.iter().filter(|entry| !entry.is_null()) {
                let entry = entry
                    .as_object()
                    .ok_or_else(|| invalid("player.inventory entry"))?;
                let item = required_string(entry, "item_id", "player.inventory item_id")?;
                let count = unsigned(
                    required_value(entry, "count", "player.inventory count")?,
                    "player.inventory count",
                )?;
                if count == 0 {
                    return Err(invalid("player.inventory count"));
                }
                evidence.push(self.fact(EvidenceFact::ItemOwned {
                    item_id: item_id(item)?,
                    count,
                }));
            }
        }
        if let Some(legendary) = optional_field_object(
            player,
            "legendary_fish_caught",
            "player.legendary_fish_caught",
        )? {
            for (item, caught) in legendary {
                let caught = caught
                    .as_bool()
                    .ok_or_else(|| invalid("player.legendary_fish_caught"))?;
                if caught {
                    evidence.push(self.fact(EvidenceFact::ItemOwned {
                        item_id: item_id(item)?,
                        count: 1,
                    }));
                }
            }
        }
        Ok(())
    }

    fn extract_npcs(
        &self,
        npcs: &Map<String, Value>,
        evidence: &mut Vec<ImportedEvidence>,
    ) -> Result<(), EvidenceError> {
        for (npc, details) in npcs {
            let npc_id = npc_id(npc)?;
            let details = details.as_object().ok_or_else(|| invalid("npcs entry"))?;
            if let Some(had_arrived) = details.get("had_arrived") {
                if had_arrived
                    .as_bool()
                    .ok_or_else(|| invalid("npcs.had_arrived"))?
                {
                    evidence.push(self.fact(EvidenceFact::NpcMet {
                        npc_id: npc_id.clone(),
                    }));
                }
            }
            if let Some(gifts) = optional_field_object(details, "gifts_given", "npcs.gifts_given")?
            {
                for (item, count) in gifts {
                    if unsigned(count, "npcs.gifts_given count")? > 0 {
                        evidence.push(self.fact(EvidenceFact::GiftKnown {
                            npc_id: npc_id.clone(),
                            item_id: item_id(item)?,
                            reaction: None,
                        }));
                    }
                }
            }
            if let Some(preferences) = optional_field_object(
                details,
                "known_gift_preferences",
                "npcs.known_gift_preferences",
            )? {
                for (item, preference) in preferences {
                    evidence.push(self.fact(EvidenceFact::GiftKnown {
                        npc_id: npc_id.clone(),
                        item_id: item_id(item)?,
                        reaction: Some(reaction(preference)?),
                    }));
                }
            }
        }
        Ok(())
    }

    fn extract_museum(
        &self,
        gamedata: &Map<String, Value>,
        evidence: &mut Vec<ImportedEvidence>,
    ) -> Result<(), EvidenceError> {
        let Some(progress) =
            optional_field_object(gamedata, "museum_progress", "gamedata.museum_progress")?
        else {
            return Ok(());
        };
        for (set_id, items) in progress {
            let items = items
                .as_object()
                .ok_or_else(|| invalid("gamedata.museum_progress set"))?;
            for (item, donated) in items {
                if donated
                    .as_bool()
                    .ok_or_else(|| invalid("gamedata.museum_progress item"))?
                {
                    evidence.push(self.fact(EvidenceFact::MuseumDonated {
                        item_id: item_id(item)?,
                        set_id: set_id.clone(),
                    }));
                }
            }
        }
        Ok(())
    }

    fn extract_collections(
        &self,
        stats: &Map<String, Value>,
        evidence: &mut Vec<ImportedEvidence>,
    ) -> Result<(), EvidenceError> {
        for collection in ["fish_caught", "bugs_caught", "gifts_given"] {
            if let Some(value) = stats.get(collection) {
                evidence.push(self.fact(EvidenceFact::CollectionKnown {
                    collection: collection.to_owned(),
                    count: unsigned(value, "game_stats count")?,
                }));
            }
        }
        Ok(())
    }

    fn fact(&self, fact: EvidenceFact) -> ImportedEvidence {
        ImportedEvidence {
            profile_id: self.profile_id.clone(),
            fact,
        }
    }
}

fn required_object<'a>(
    vault: &'a Vault,
    section: &'static str,
) -> Result<&'a Map<String, Value>, EvidenceError> {
    vault
        .section(section)
        .ok_or(EvidenceError::MissingSection(section))?
        .as_object()
        .ok_or_else(|| invalid(section))
}

fn optional_object<'a>(
    vault: &'a Vault,
    section: &'static str,
) -> Result<Option<&'a Map<String, Value>>, EvidenceError> {
    vault
        .section(section)
        .map(|value| value.as_object().ok_or_else(|| invalid(section)))
        .transpose()
}

fn optional_field_object<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    path: &str,
) -> Result<Option<&'a Map<String, Value>>, EvidenceError> {
    object
        .get(key)
        .map(|value| value.as_object().ok_or_else(|| invalid(path)))
        .transpose()
}

fn game_version(info: &Map<String, Value>) -> Result<String, EvidenceError> {
    let version = required_value(info, "version", "info.version")?;
    if let Some(version) = version.as_str() {
        return Ok(version.to_owned());
    }

    let version = version.as_object().ok_or_else(|| invalid("info.version"))?;
    let major = unsigned(
        required_value(version, "major", "info.version.major")?,
        "info.version.major",
    )?;
    let minor = unsigned(
        required_value(version, "minor", "info.version.minor")?,
        "info.version.minor",
    )?;
    let patch = unsigned(
        required_value(version, "patch", "info.version.patch")?,
        "info.version.patch",
    )?;
    match version.get("pre") {
        None | Some(Value::Null) => Ok(format!("{major}.{minor}.{patch}")),
        Some(Value::String(pre)) if !pre.is_empty() => Ok(format!("{major}.{minor}.{patch}-{pre}")),
        _ => Err(invalid("info.version.pre")),
    }
}

fn required_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    path: &str,
) -> Result<&'a str, EvidenceError> {
    required_value(object, key, path)?
        .as_str()
        .ok_or_else(|| invalid(path))
}

fn required_value<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    path: &str,
) -> Result<&'a Value, EvidenceError> {
    object.get(key).ok_or_else(|| invalid(path))
}

fn unsigned(value: &Value, path: &str) -> Result<u64, EvidenceError> {
    value.as_u64().ok_or_else(|| invalid(path))
}

fn item_id(value: &str) -> Result<ItemId, EvidenceError> {
    ItemId::new(value).map_err(|_| invalid("item identifier"))
}

fn npc_id(value: &str) -> Result<NpcId, EvidenceError> {
    NpcId::new(value).map_err(|_| invalid("npc identifier"))
}

fn reaction(value: &Value) -> Result<GiftReaction, EvidenceError> {
    match value.as_str() {
        Some("loved") => Ok(GiftReaction::Loved),
        Some("liked") => Ok(GiftReaction::Liked),
        Some("neutral") => Ok(GiftReaction::Neutral),
        Some("disliked") => Ok(GiftReaction::Disliked),
        Some("hated") => Ok(GiftReaction::Hated),
        _ => Err(invalid("npcs.known_gift_preferences")),
    }
}

fn invalid(path: impl Into<String>) -> EvidenceError {
    EvidenceError::InvalidShape(path.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compatibility::matrix::CompatibilityMatrix,
        save::vault::VaultReader,
        test_support::vault::{sections, vault_bytes, verified_matrix},
    };
    use serde_json::json;

    fn extract(
        parts: &[(&str, serde_json::Value)],
        matrix: &CompatibilityMatrix,
    ) -> Result<Vec<ImportedEvidence>, EvidenceError> {
        let vault = VaultReader::read(vault_bytes(parts).as_slice()).unwrap();
        EvidenceExtractor::new(ProfileId::new("123").unwrap(), matrix).extract(&vault)
    }

    #[test]
    fn extracts_only_explicit_whitelisted_evidence_for_the_given_profile() {
        let result = extract(&sections(), &verified_matrix()).unwrap();
        assert_eq!(result.len(), 11);
        assert!(result
            .iter()
            .all(|event| event.profile_id.as_str() == "123"));
        let values: Vec<_> = result
            .iter()
            .map(|event| serde_json::to_value(&event.fact).unwrap())
            .collect();
        for expected in [
            json!({"type":"profile_seen","name":"Ari","farm_name":"Test"}),
            json!({"type":"item_owned","item_id":"test_ore","count":2}),
            json!({"type":"item_owned","item_id":"test_seed","count":3}),
            json!({"type":"item_owned","item_id":"test_legendary","count":1}),
            json!({"type":"npc_met","npc_id":"test_npc"}),
            json!({"type":"gift_known","npc_id":"test_npc","item_id":"test_seed","reaction":null}),
            json!({"type":"gift_known","npc_id":"test_npc","item_id":"test_ore","reaction":"loved"}),
            json!({"type":"museum_donated","item_id":"test_ore","set_id":"test_set"}),
            json!({"type":"collection_known","collection":"fish_caught","count":4}),
            json!({"type":"collection_known","collection":"bugs_caught","count":2}),
            json!({"type":"collection_known","collection":"gifts_given","count":1}),
        ] {
            assert!(values.contains(&expected), "missing {expected}");
        }
        let text = serde_json::to_string(&result).unwrap();
        for secret in [
            "secret_item",
            "unknown_npc",
            "unowned_item",
            "undonated_item",
            "uncaught_fish",
        ] {
            assert!(!text.contains(secret));
        }
    }

    #[test]
    fn extracts_deduplicated_items_from_the_1_0_4_array_format() {
        let matrix = serde_json::from_value(json!({"records":[{
            "game_version":"1.0.4", "event_schema_versions":[1], "companion_versions":[],
            "save_parser_versions":[1], "catalog_parser_versions":[], "probe_required":false
        }]}))
        .unwrap();
        let parts = [
            (
                "info",
                json!({"version":{"major":1,"minor":0,"patch":4,"pre":null}}),
            ),
            ("header", json!({"name":"Ari","farm_name":"Test"})),
            (
                "player",
                json!({"items_acquired":["test_ore","test_seed","test_ore"]}),
            ),
        ];

        let values: Vec<_> = extract(&parts, &matrix)
            .unwrap()
            .into_iter()
            .map(|event| serde_json::to_value(event.fact).unwrap())
            .collect();
        assert_eq!(
            values,
            vec![
                json!({"type":"profile_seen","name":"Ari","farm_name":"Test"}),
                json!({"type":"item_owned","item_id":"test_ore","count":1}),
                json!({"type":"item_owned","item_id":"test_seed","count":1}),
            ]
        );
    }

    #[test]
    fn v1_0_4_uses_only_the_verified_acquisition_list() {
        let matrix = serde_json::from_value(json!({"records":[{
            "game_version":"1.0.4", "event_schema_versions":[1], "companion_versions":[],
            "save_parser_versions":[1], "catalog_parser_versions":[], "probe_required":false
        }]}))
        .unwrap();
        let parts = [
            (
                "info",
                json!({"version":{"major":1,"minor":0,"patch":4,"pre":null}}),
            ),
            ("header", json!({"name":"Ari","farm_name":"Test"})),
            (
                "player",
                json!({
                    "items_acquired":["test_ore"],
                    "inventory":[{"different_1_0_4_shape":true}]
                }),
            ),
        ];

        let values: Vec<_> = extract(&parts, &matrix)
            .unwrap()
            .into_iter()
            .map(|event| serde_json::to_value(event.fact).unwrap())
            .collect();
        assert_eq!(values.len(), 2);
        assert_eq!(
            values[1],
            json!({"type":"item_owned","item_id":"test_ore","count":1})
        );
    }

    #[test]
    fn embedded_1_0_5_parser_uses_the_same_narrow_verified_surface() {
        let parts = [
            (
                "info",
                json!({"version":{"major":1,"minor":0,"patch":5,"pre":null}}),
            ),
            ("header", json!({"name":"Ari","farm_name":"Test"})),
            (
                "player",
                json!({
                    "items_acquired":["test_ore"],
                    "inventory":[{"unapproved_shape":true}]
                }),
            ),
        ];

        let values: Vec<_> = extract(&parts, &CompatibilityMatrix::embedded().unwrap())
            .unwrap()
            .into_iter()
            .map(|event| serde_json::to_value(event.fact).unwrap())
            .collect();

        assert_eq!(
            values,
            vec![
                json!({"type":"profile_seen","name":"Ari","farm_name":"Test"}),
                json!({"type":"item_owned","item_id":"test_ore","count":1}),
            ]
        );
    }

    #[test]
    fn unknown_or_live_only_versions_and_wrong_parser_versions_are_rejected() {
        let mut parts = sections();
        parts[0].1 = json!({"version":"9.9.9"});
        assert!(matches!(
            extract(&parts, &verified_matrix()),
            Err(EvidenceError::UnsupportedVersion(_))
        ));
        parts[0].1 = json!({"version":"1.0.4"});
        parts[2].1 = json!({"items_acquired":["test_ore"]});
        assert!(extract(&parts, &CompatibilityMatrix::embedded().unwrap()).is_ok());
        let wrong = serde_json::from_value(json!({"records":[{"game_version":"synthetic-1","event_schema_versions":[1],"companion_versions":[],"save_parser_versions":[2],"catalog_parser_versions":[1],"probe_required":false}]})).unwrap();
        assert!(matches!(
            extract(&sections(), &wrong),
            Err(EvidenceError::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn malformed_known_fields_fail_the_entire_import_but_unknown_fields_are_ignored() {
        for (section, value) in [
            ("player", json!({"items_acquired":{"test_ore":-1}})),
            (
                "player",
                json!({"inventory":[{"item_id":"test_ore","count":0}]}),
            ),
            (
                "player",
                json!({"legendary_fish_caught":{"test_fish":"true"}}),
            ),
            ("npcs", json!({"test_npc":{"had_arrived":1}})),
            (
                "npcs",
                json!({"test_npc":{"known_gift_preferences":{"test_ore":"amazing"}}}),
            ),
            ("gamedata", json!({"museum_progress":["test_ore"]})),
            ("game_stats", json!({"fish_caught":1.5})),
        ] {
            let mut parts = sections();
            parts
                .iter_mut()
                .find(|(name, _)| *name == section)
                .unwrap()
                .1 = value;
            assert!(
                matches!(
                    extract(&parts, &verified_matrix()),
                    Err(EvidenceError::InvalidShape(_))
                ),
                "accepted {section}"
            );
        }
        let minimal = [
            ("info", json!({"version":"synthetic-1"})),
            (
                "header",
                json!({"name":"Ari","farm_name":"Test","story_flag":true}),
            ),
            ("unverified_section", json!({"items":["secret_item"]})),
        ];
        assert_eq!(extract(&minimal, &verified_matrix()).unwrap().len(), 1);
        assert!(extract(&[], &verified_matrix()).is_err());
    }
}
