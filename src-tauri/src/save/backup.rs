use crate::{
    compatibility::CompatibilityMatrix,
    domain::{ItemId, ProfileId},
    save::{
        evidence::{EvidenceError, EvidenceExtractor, EvidenceFact},
        vault::{VaultError, VaultReader},
    },
};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ExistingDiscoveries {
    pub source_hash: [u8; 32],
    pub profile_id: ProfileId,
    pub game_version: String,
    pub items: Vec<ItemId>,
    pub journal: crate::journal::evidence::JournalEvidence,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveImportError {
    #[error("selected save filename does not contain a numeric Fields of Mistria profile id")]
    MissingProfileId,
    #[error("could not read the selected save: {0}")]
    Read(#[from] std::io::Error),
    #[error(transparent)]
    Vault(#[from] VaultError),
    #[error(transparent)]
    Evidence(#[from] EvidenceError),
}

/// Extracts approved discoveries from bytes which have already been copied into
/// tracker-owned storage. The caller supplies the companion-confirmed profile;
/// a snapshot filename is intentionally never treated as profile evidence.
pub fn discoveries_from_bytes(
    bytes: &[u8],
    profile_id: ProfileId,
    compatibility: &CompatibilityMatrix,
) -> Result<ExistingDiscoveries, SaveImportError> {
    let source_hash: [u8; 32] = Sha256::digest(&bytes).into();
    let vault = VaultReader::read(bytes)?;
    let game_version = EvidenceExtractor::game_version(&vault)?;
    let evidence = EvidenceExtractor::new(profile_id.clone(), compatibility).extract(&vault)?;
    let journal = if game_version == "1.0.4" {
        crate::journal::evidence::JournalEvidence::from_vault(&vault)?
    } else {
        Default::default()
    };
    let items = evidence
        .into_iter()
        .filter_map(|evidence| match evidence.fact {
            EvidenceFact::ItemOwned { item_id, .. } => Some(item_id),
            _ => None,
        })
        .collect();
    Ok(ExistingDiscoveries {
        source_hash,
        profile_id,
        game_version,
        items,
        journal,
    })
}

pub(crate) fn profile_id_from_save_filename(path: &Path) -> Result<ProfileId, SaveImportError> {
    let filename = path
        .file_name()
        .and_then(|filename| filename.to_str())
        .ok_or(SaveImportError::MissingProfileId)?;
    let (_, suffix) = filename
        .rsplit_once("game-")
        .ok_or(SaveImportError::MissingProfileId)?;
    let profile_id = suffix
        .split('-')
        .next()
        .ok_or(SaveImportError::MissingProfileId)?;
    ProfileId::new(profile_id).map_err(|_| SaveImportError::MissingProfileId)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::vault::vault_bytes;
    use serde_json::json;
    #[test]
    fn extracts_discoveries_from_a_tracker_owned_snapshot_using_the_companion_profile() {
        let bytes = vault_bytes(&[
            (
                "info",
                json!({"version":{"major":1,"minor":0,"patch":4,"pre":null}}),
            ),
            ("header", json!({"name":"Ari","farm_name":"Test"})),
            ("player", json!({"items_acquired":["test_ore"]})),
        ]);

        let discoveries = discoveries_from_bytes(
            &bytes,
            ProfileId::new("331655283").unwrap(),
            &CompatibilityMatrix::embedded().unwrap(),
        )
        .unwrap();

        assert_eq!(discoveries.profile_id.as_str(), "331655283");
        assert_eq!(discoveries.items, vec![ItemId::new("test_ore").unwrap()]);
    }
}
