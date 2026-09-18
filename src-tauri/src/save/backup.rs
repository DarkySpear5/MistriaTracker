use crate::{
    compatibility::CompatibilityMatrix,
    domain::{ItemId, ProfileId},
    save::{
        evidence::{EvidenceError, EvidenceExtractor, EvidenceFact},
        vault::{VaultError, VaultReader},
    },
};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[derive(Clone, Debug)]
pub struct ExistingDiscoveries {
    pub source_hash: [u8; 32],
    pub profile_id: ProfileId,
    pub game_version: String,
    pub items: Vec<ItemId>,
    pub journal: crate::journal::evidence::JournalEvidence,
}

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("backup is not a regular .sav file")]
    InvalidBackup,
    #[error("backup filename does not contain a numeric Fields of Mistria profile id")]
    MissingProfileId,
    #[error("backup exceeds the Tracker safety limit")]
    InputTooLarge,
    #[error("could not read the backup: {0}")]
    Read(#[from] std::io::Error),
    #[error(transparent)]
    Vault(#[from] VaultError),
    #[error(transparent)]
    Evidence(#[from] EvidenceError),
}

/// Reads a user-provided backup copy into memory and extracts only the
/// compatibility-approved discovery records. This function never writes the
/// backup or opens a live game-save path.
pub fn existing_discoveries(
    backup: &Path,
    compatibility: &CompatibilityMatrix,
) -> Result<ExistingDiscoveries, BackupError> {
    let metadata = fs::metadata(backup)?;
    if backup.extension().and_then(|extension| extension.to_str()) != Some("sav")
        || !metadata.is_file()
    {
        return Err(BackupError::InvalidBackup);
    }
    if metadata.len() > crate::save::vault::MAX_COMPRESSED_BYTES as u64 {
        return Err(BackupError::InputTooLarge);
    }
    let bytes = fs::read(backup)?;
    discoveries_from_bytes(
        &bytes,
        profile_id_from_backup_filename(backup)?,
        compatibility,
    )
}

/// Extracts approved discoveries from bytes which have already been copied into
/// tracker-owned storage. The caller supplies the companion-confirmed profile;
/// a snapshot filename is intentionally never treated as profile evidence.
pub fn discoveries_from_bytes(
    bytes: &[u8],
    profile_id: ProfileId,
    compatibility: &CompatibilityMatrix,
) -> Result<ExistingDiscoveries, BackupError> {
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

fn profile_id_from_backup_filename(path: &Path) -> Result<ProfileId, BackupError> {
    let filename = path
        .file_name()
        .and_then(|filename| filename.to_str())
        .ok_or(BackupError::MissingProfileId)?;
    let (_, suffix) = filename
        .rsplit_once("game-")
        .ok_or(BackupError::MissingProfileId)?;
    let profile_id = suffix
        .split('-')
        .next()
        .ok_or(BackupError::MissingProfileId)?;
    ProfileId::new(profile_id).map_err(|_| BackupError::MissingProfileId)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::vault::vault_bytes;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn extracts_only_items_from_a_version_approved_backup_copy() {
        let directory = tempdir().unwrap();
        let backup = directory.path().join("Ari-game-1849811906-1.sav");
        fs::write(
            &backup,
            vault_bytes(&[
                (
                    "info",
                    json!({"version":{"major":1,"minor":0,"patch":4,"pre":null}}),
                ),
                ("header", json!({"name":"Ari","farm_name":"Test"})),
                (
                    "player",
                    json!({"items_acquired":["test_ore","test_seed","test_ore"]}),
                ),
            ]),
        )
        .unwrap();

        let discoveries =
            existing_discoveries(&backup, &CompatibilityMatrix::embedded().unwrap()).unwrap();
        assert_eq!(discoveries.profile_id.as_str(), "1849811906");
        assert_eq!(discoveries.game_version, "1.0.4");
        assert_eq!(
            discoveries
                .items
                .iter()
                .map(ItemId::as_str)
                .collect::<Vec<_>>(),
            vec!["test_ore", "test_seed"]
        );
    }

    #[test]
    fn rejects_a_backup_filename_without_a_profile_id() {
        let directory = tempdir().unwrap();
        let backup = directory.path().join("copy.sav");
        fs::write(&backup, b"not read because the name is invalid").unwrap();
        assert!(matches!(
            existing_discoveries(&backup, &CompatibilityMatrix::embedded().unwrap()),
            Err(BackupError::MissingProfileId)
        ));
    }

    #[test]
    fn rejects_an_oversized_backup_before_parsing_it() {
        let directory = tempdir().unwrap();
        let backup = directory.path().join("Ari-game-1849811906-1.sav");
        fs::write(&backup, vec![0; 16 * 1024 * 1024 + 1]).unwrap();

        assert_eq!(
            existing_discoveries(&backup, &CompatibilityMatrix::embedded().unwrap())
                .unwrap_err()
                .to_string(),
            "backup exceeds the Tracker safety limit"
        );
    }

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
