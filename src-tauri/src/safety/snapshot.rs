use super::paths::GameSavePath;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime},
};
use tempfile::NamedTempFile;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub path: PathBuf,
    pub manifest_path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("could not read the game save: {0}")]
    Read(#[source] std::io::Error),
    #[error("could not create tracker backup storage: {0}")]
    BackupStorage(#[source] std::io::Error),
    #[error("the game save changed while the tracker was reading it")]
    SourceChanged,
    #[error("could not write a tracker-owned snapshot: {0}")]
    SnapshotWrite(#[source] std::io::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceFingerprint {
    length: u64,
    modified: SystemTime,
}

pub struct SnapshotService {
    pub(crate) backups: PathBuf,
    stability_window: Duration,
}

impl SnapshotService {
    /// Stores temporary copies only in tracker-local data, never beside a game save.
    pub fn for_local_data(local_root: &Path) -> Result<Self, SnapshotError> {
        Self::with_storage(local_root, Duration::from_millis(80))
    }

    #[cfg(test)]
    pub fn for_fixture(
        local_root: &Path,
        stability_window: Duration,
    ) -> Result<Self, SnapshotError> {
        Self::with_storage(local_root, stability_window)
    }

    fn with_storage(local_root: &Path, stability_window: Duration) -> Result<Self, SnapshotError> {
        let backups = local_root.join("MistriaTracker/backups/game-saves");
        fs::create_dir_all(&backups).map_err(SnapshotError::BackupStorage)?;
        Ok(Self {
            backups: fs::canonicalize(backups).map_err(SnapshotError::BackupStorage)?,
            stability_window,
        })
    }

    pub fn snapshot(&self, source: &GameSavePath) -> Result<Snapshot, SnapshotError> {
        let before = source_fingerprint(source.as_path())?;
        if !self.stability_window.is_zero() {
            thread::sleep(self.stability_window);
        }
        if source_fingerprint(source.as_path())? != before {
            return Err(SnapshotError::SourceChanged);
        }

        let mut input = OpenOptions::new()
            .read(true)
            .open(source.as_path())
            .map_err(SnapshotError::Read)?;
        let mut temporary =
            NamedTempFile::new_in(&self.backups).map_err(SnapshotError::BackupStorage)?;
        copy_to_snapshot(&mut input, temporary.as_file_mut())?;

        if source_fingerprint(source.as_path())? != before {
            return Err(SnapshotError::SourceChanged);
        }

        let bytes = fs::read(temporary.path()).map_err(SnapshotError::SnapshotWrite)?;
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        let path = self.backups.join(format!("{}.sav", Uuid::new_v4()));
        temporary
            .persist(&path)
            .map_err(|error| SnapshotError::SnapshotWrite(error.error))?;
        let manifest_path = path.with_extension("manifest.json");
        let manifest = serde_json::json!({"sha256": sha256, "source_size": before.length});
        fs::write(
            &manifest_path,
            serde_json::to_vec(&manifest).expect("snapshot manifest is serializable"),
        )
        .map_err(SnapshotError::SnapshotWrite)?;

        Ok(Snapshot {
            path,
            manifest_path,
            sha256,
        })
    }
}

fn source_fingerprint(path: &Path) -> Result<SourceFingerprint, SnapshotError> {
    let metadata = fs::metadata(path).map_err(SnapshotError::Read)?;
    if !metadata.is_file() {
        return Err(SnapshotError::SourceChanged);
    }
    Ok(SourceFingerprint {
        length: metadata.len(),
        modified: metadata.modified().map_err(SnapshotError::Read)?,
    })
}

fn copy_to_snapshot(input: &mut File, output: &mut File) -> Result<(), SnapshotError> {
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = input.read(&mut buffer).map_err(SnapshotError::Read)?;
        if count == 0 {
            break;
        }
        output
            .write_all(&buffer[..count])
            .map_err(SnapshotError::SnapshotWrite)?;
    }
    output.flush().map_err(SnapshotError::SnapshotWrite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        safety::paths::GameSavePath, save::vault::VaultReader, test_support::vault::Fixture,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::Duration,
    };

    #[test]
    fn snapshot_never_changes_source_and_parser_reads_named_json_sections() {
        let fixture = Fixture::new();
        let before = fs::read(&fixture.source).unwrap();
        let metadata = fs::metadata(&fixture.source).unwrap();
        let service = SnapshotService::for_fixture(&fixture.local, Duration::ZERO).unwrap();
        let source = GameSavePath::new(&fixture.saves, &fixture.source).unwrap();
        let snapshot = service.snapshot(&source).unwrap();
        assert_eq!(before, fs::read(&fixture.source).unwrap());
        assert_eq!(
            metadata.modified().unwrap(),
            fs::metadata(&fixture.source).unwrap().modified().unwrap()
        );
        assert_ne!(fixture.source, snapshot.path);
        assert!(snapshot.path.starts_with(
            fs::canonicalize(fixture.local.join("MistriaTracker/backups/game-saves")).unwrap()
        ));
        assert_eq!(snapshot.sha256, format!("{:x}", Sha256::digest(&before)));
        assert_eq!(fs::read(&snapshot.path).unwrap(), before);
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&snapshot.manifest_path).unwrap()).unwrap();
        assert_eq!(manifest["sha256"], snapshot.sha256);
        assert_eq!(
            VaultReader::read(fs::File::open(&snapshot.path).unwrap())
                .unwrap()
                .section("header")
                .unwrap()["name"],
            "Ari"
        );
        let another = service.snapshot(&source).unwrap();
        assert_ne!(snapshot.path, another.path);
        assert!(snapshot.path.exists());
    }

    #[test]
    fn changing_source_is_rejected_without_finalizing_a_copy() {
        let fixture = Fixture::new();
        let service =
            SnapshotService::for_fixture(&fixture.local, Duration::from_millis(100)).unwrap();
        let source = GameSavePath::new(&fixture.saves, &fixture.source).unwrap();
        let running = Arc::new(AtomicBool::new(true));
        let ready = Arc::new(AtomicBool::new(false));
        let thread = {
            let path = fixture.source.clone();
            let running = running.clone();
            let ready = ready.clone();
            std::thread::spawn(move || {
                use std::io::Write;
                let mut file = fs::OpenOptions::new().append(true).open(path).unwrap();
                while running.load(Ordering::SeqCst) {
                    file.write_all(b"synthetic change").unwrap();
                    file.flush().unwrap();
                    ready.store(true, Ordering::SeqCst);
                    std::thread::sleep(Duration::from_millis(1));
                }
            })
        };
        while !ready.load(Ordering::SeqCst) {
            std::thread::yield_now();
        }
        let result = service.snapshot(&source);
        running.store(false, Ordering::SeqCst);
        thread.join().unwrap();
        assert!(matches!(result, Err(SnapshotError::SourceChanged)));
        assert_eq!(fs::read_dir(&service.backups).unwrap().count(), 0);
    }

    #[test]
    fn read_only_source_can_be_snapshotted() {
        let fixture = Fixture::new();
        let original = fs::metadata(&fixture.source).unwrap().permissions();
        let mut readonly = original.clone();
        readonly.set_readonly(true);
        fs::set_permissions(&fixture.source, readonly).unwrap();
        let result = SnapshotService::for_fixture(&fixture.local, Duration::ZERO)
            .unwrap()
            .snapshot(&GameSavePath::new(&fixture.saves, &fixture.source).unwrap());
        fs::set_permissions(&fixture.source, original).unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn production_snapshot_storage_is_created_only_under_tracker_local_data() {
        let fixture = Fixture::new();
        let service = SnapshotService::for_local_data(&fixture.local).unwrap();

        assert!(service
            .backups
            .ends_with("MistriaTracker/backups/game-saves"));
        assert!(service.backups.exists());
    }

    #[test]
    fn replaced_source_after_validation_is_rejected() {
        let fixture = Fixture::new();
        let source = GameSavePath::new(&fixture.saves, &fixture.source).unwrap();
        fs::remove_file(&fixture.source).unwrap();
        fs::create_dir(&fixture.source).unwrap();
        let service = SnapshotService::for_fixture(&fixture.local, Duration::ZERO).unwrap();
        assert!(service.snapshot(&source).is_err());
    }
}
