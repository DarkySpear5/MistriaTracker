use super::{migrations, RepoError, Repository};
use rusqlite::Connection;
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use uuid::Uuid;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, RepoError> {
        let parent = path.parent().ok_or(RepoError::InvalidDatabasePath)?;
        fs::create_dir_all(parent)?;
        let backup = backup_existing_database(path)?;
        let connection = Connection::open(path)?;
        if let Err(error) = migrations::apply(&connection) {
            drop(connection);
            if let Some(backup) = backup {
                fs::copy(backup, path)?;
            }
            return Err(error.into());
        }
        Ok(Self { connection })
    }

    #[cfg(test)]
    pub(crate) fn open_with_migration_for_test(path: &Path, sql: &str) -> Result<Self, RepoError> {
        Self::open_with_migration(path, sql)
    }

    #[cfg(test)]
    fn open_with_migration(path: &Path, sql: &str) -> Result<Self, RepoError> {
        let parent = path.parent().ok_or(RepoError::InvalidDatabasePath)?;
        fs::create_dir_all(parent)?;
        let backup = backup_existing_database(path)?;
        let connection = Connection::open(path)?;
        if let Err(error) = migrations::apply_sql(&connection, sql) {
            drop(connection);
            if let Some(backup) = backup {
                fs::copy(backup, path)?;
            }
            return Err(error.into());
        }
        Ok(Self { connection })
    }

    pub fn into_repository(self) -> Repository {
        Repository::from_connection(self.connection)
    }
}

fn backup_existing_database(path: &Path) -> Result<Option<PathBuf>, RepoError> {
    if !path.is_file() {
        return Ok(None);
    }

    let checkpoint = Connection::open(path)?;
    checkpoint.execute_batch("PRAGMA wal_checkpoint(FULL);")?;
    drop(checkpoint);

    let backup_directory = path
        .parent()
        .ok_or(RepoError::InvalidDatabasePath)?
        .join("backups/tracker-db");
    fs::create_dir_all(&backup_directory)?;
    let backup = backup_directory.join(format!("{}.sqlite", Uuid::new_v4()));
    fs::copy(path, &backup)?;
    retain_latest_backups(&backup_directory)?;
    Ok(Some(backup))
}

fn retain_latest_backups(directory: &Path) -> Result<(), RepoError> {
    let mut backups: Vec<_> = fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("sqlite"))
        .collect();
    backups.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });
    for backup in backups.into_iter().rev().skip(10) {
        fs::remove_file(backup)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use tempfile::tempdir;

    #[test]
    fn failed_migration_restores_tracker_database_only() {
        let directory = tempdir().unwrap();
        let database_path = directory.path().join("MistriaTracker/tracker.sqlite");
        drop(Database::open(&database_path).unwrap());
        let before = hash(&database_path);

        assert!(Database::open_with_migration_for_test(
            &database_path,
            "CREATE TABLE migration_probe (id INTEGER); NOT VALID SQL;"
        )
        .is_err());

        assert_eq!(hash(&database_path), before);
        let backups = directory.path().join("MistriaTracker/backups/tracker-db");
        assert_eq!(std::fs::read_dir(backups).unwrap().count(), 1);
    }

    fn hash(path: &Path) -> String {
        format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
    }
}
