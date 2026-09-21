#[cfg(test)]
use super::migrations;
use crate::domain::{
    CompanionEvent, EntityId, EventEnvelope, ItemId, ProfileId, EVENT_SCHEMA_VERSION,
};
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcceptResult {
    Inserted,
    Duplicate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportResult {
    Inserted,
    Duplicate,
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("tracker database path has no parent directory")]
    InvalidDatabasePath,
    #[error("could not access tracker-owned storage: {0}")]
    Io(#[from] std::io::Error),
    #[error("tracker database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("event serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("tracker database contains an invalid profile identifier")]
    InvalidProfileId,
    #[error("notes cannot exceed 20,000 characters")]
    NoteTooLong,
}

pub struct Repository {
    connection: Connection,
}

impl Repository {
    pub(crate) fn from_connection(connection: Connection) -> Self {
        Self { connection }
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self, RepoError> {
        let connection = Connection::open_in_memory()?;
        migrations::apply(&connection)?;
        Ok(Self::from_connection(connection))
    }

    pub fn accept_event(&mut self, event: &EventEnvelope) -> Result<AcceptResult, RepoError> {
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT OR IGNORE INTO profiles (id) VALUES (?1)",
            params![event.profile_id.as_str()],
        )?;
        let event_json = serde_json::to_string(event)?;
        let inserted = transaction.execute(
            "INSERT OR IGNORE INTO accepted_events (session_id, sequence, profile_id, event_json) VALUES (?1, ?2, ?3, ?4)",
            params![event.session_id.to_string(), event.sequence, event.profile_id.as_str(), event_json],
        )?;
        transaction.commit()?;
        Ok(if inserted == 1 {
            AcceptResult::Inserted
        } else {
            AcceptResult::Duplicate
        })
    }

    /// Records a verified backup's acquired-item evidence in tracker-owned
    /// storage. The source hash makes retries idempotent; the backup itself is
    /// never modified.
    pub fn import_backup_items(
        &mut self,
        source_hash: [u8; 32],
        profile_id: &ProfileId,
        game_version: &str,
        items: &[ItemId],
    ) -> Result<ImportResult, RepoError> {
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT OR IGNORE INTO profiles (id) VALUES (?1)",
            params![profile_id.as_str()],
        )?;
        let source_sha256 = hex(&source_hash);
        let inserted = transaction.execute(
            "INSERT OR IGNORE INTO imported_backups (source_sha256, profile_id, game_version, imported_at) VALUES (?1, ?2, ?3, datetime('now'))",
            params![source_sha256, profile_id.as_str(), game_version],
        )?;
        if inserted == 0 {
            transaction.execute(
                "INSERT INTO settings (key, value) VALUES ('active_profile', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![profile_id.as_str()],
            )?;
            transaction.commit()?;
            return Ok(ImportResult::Duplicate);
        }

        let session_id = imported_session_id(source_hash);
        for (index, item_id) in items.iter().enumerate() {
            let event = EventEnvelope {
                schema_version: EVENT_SCHEMA_VERSION,
                companion_version: "backup-import-1".to_owned(),
                game_version: game_version.to_owned(),
                profile_id: profile_id.clone(),
                session_id,
                sequence: (index + 1) as u64,
                save_file: None,
                event: CompanionEvent::ItemObtained {
                    item_id: item_id.clone(),
                    count: 1,
                },
            };
            transaction.execute(
                "INSERT OR IGNORE INTO accepted_events (session_id, sequence, profile_id, event_json) VALUES (?1, ?2, ?3, ?4)",
                params![
                    event.session_id.to_string(),
                    event.sequence,
                    profile_id.as_str(),
                    serde_json::to_string(&event)?,
                ],
            )?;
        }
        transaction.execute(
            "INSERT INTO settings (key, value) VALUES ('active_profile', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![profile_id.as_str()],
        )?;
        transaction.commit()?;
        Ok(ImportResult::Inserted)
    }

    /// Replaces the current progress projected from one confirmed save while
    /// preserving profile-scoped notes and every other profile. This is used
    /// when a newly loaded game session makes an older on-disk save
    /// authoritative again after unsaved live events.
    pub fn replace_save_state(
        &mut self,
        source_hash: [u8; 32],
        profile_id: &ProfileId,
        game_version: &str,
        items: &[ItemId],
        journal_evidence_json: &str,
    ) -> Result<(), RepoError> {
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT OR IGNORE INTO profiles (id) VALUES (?1)",
            params![profile_id.as_str()],
        )?;
        transaction.execute(
            "DELETE FROM accepted_events WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?;
        transaction.execute(
            "DELETE FROM imported_backups WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?;

        let source_sha256 = hex(&source_hash);
        transaction.execute(
            "INSERT INTO imported_backups (source_sha256, profile_id, game_version, imported_at) VALUES (?1, ?2, ?3, datetime('now'))",
            params![source_sha256, profile_id.as_str(), game_version],
        )?;
        let session_id = imported_session_id(source_hash);
        for (index, item_id) in items.iter().enumerate() {
            let event = EventEnvelope {
                schema_version: EVENT_SCHEMA_VERSION,
                companion_version: "save-reconciliation-1".to_owned(),
                game_version: game_version.to_owned(),
                profile_id: profile_id.clone(),
                session_id,
                sequence: (index + 1) as u64,
                save_file: None,
                event: CompanionEvent::ItemObtained {
                    item_id: item_id.clone(),
                    count: 1,
                },
            };
            transaction.execute(
                "INSERT INTO accepted_events (session_id, sequence, profile_id, event_json) VALUES (?1, ?2, ?3, ?4)",
                params![
                    event.session_id.to_string(),
                    event.sequence,
                    profile_id.as_str(),
                    serde_json::to_string(&event)?,
                ],
            )?;
        }
        transaction.execute(
            "INSERT INTO settings (key, value) VALUES ('active_profile', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![profile_id.as_str()],
        )?;
        let evidence_key = format!("journal_evidence_v1:{}", profile_id.as_str());
        transaction.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![evidence_key, journal_evidence_json],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn events(&self, profile_id: &ProfileId) -> Result<Vec<EventEnvelope>, RepoError> {
        let mut statement = self.connection.prepare(
            "SELECT event_json FROM accepted_events WHERE profile_id = ?1 ORDER BY rowid",
        )?;
        let event_json = statement
            .query_map(params![profile_id.as_str()], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        event_json
            .into_iter()
            .map(|json| serde_json::from_str(&json).map_err(RepoError::from))
            .collect()
    }

    pub fn activate_profile(&mut self, profile_id: &ProfileId) -> Result<(), RepoError> {
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT OR IGNORE INTO profiles (id) VALUES (?1)",
            params![profile_id.as_str()],
        )?;
        transaction.execute(
            "INSERT INTO settings (key, value) VALUES ('active_profile', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![profile_id.as_str()],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn active_profile(&self) -> Result<Option<ProfileId>, RepoError> {
        let profile_id = self
            .connection
            .query_row(
                "SELECT value FROM settings WHERE key = 'active_profile'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        profile_id
            .map(|value| ProfileId::new(value).map_err(|_| RepoError::InvalidProfileId))
            .transpose()
    }

    pub fn setting(&self, key: &str) -> Result<Option<String>, RepoError> {
        self.connection
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(RepoError::from)
    }

    pub fn save_setting(&mut self, key: &str, value: &str) -> Result<(), RepoError> {
        self.connection.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn profiles(&self) -> Result<Vec<ProfileId>, RepoError> {
        let mut statement = self
            .connection
            .prepare("SELECT id FROM profiles ORDER BY id")?;
        let profiles = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        profiles
            .into_iter()
            .map(|value| ProfileId::new(value).map_err(|_| RepoError::InvalidProfileId))
            .collect()
    }

    pub fn save_note(
        &mut self,
        profile_id: &ProfileId,
        entity_id: &EntityId,
        text: &str,
    ) -> Result<(), RepoError> {
        if text.chars().count() > 20_000 {
            return Err(RepoError::NoteTooLong);
        }
        let entity_id = serde_json::to_string(entity_id)?;
        self.connection.execute(
            "INSERT INTO notes (profile_id, entity_id, text) VALUES (?1, ?2, ?3) ON CONFLICT(profile_id, entity_id) DO UPDATE SET text = excluded.text",
            params![profile_id.as_str(), entity_id, text],
        )?;
        Ok(())
    }

    pub fn note(
        &self,
        profile_id: &ProfileId,
        entity_id: &EntityId,
    ) -> Result<Option<String>, RepoError> {
        let entity_id = serde_json::to_string(entity_id)?;
        self.connection
            .query_row(
                "SELECT text FROM notes WHERE profile_id = ?1 AND entity_id = ?2",
                params![profile_id.as_str(), entity_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(RepoError::from)
    }
}

fn imported_session_id(source_hash: [u8; 32]) -> uuid::Uuid {
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&source_hash[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
