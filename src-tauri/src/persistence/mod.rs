pub mod database;
pub mod migrations;
pub mod repository;

pub use database::Database;
pub use repository::{AcceptResult, ImportResult, RepoError, Repository};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CompanionEvent, EventEnvelope, ItemId, ProfileId};
    use uuid::Uuid;

    #[test]
    fn duplicate_event_is_idempotent_and_profiles_are_isolated() {
        let mut repo = fixture_repo();
        let first_profile = ProfileId::new("1849811906").unwrap();
        let second_profile = ProfileId::new("249165455").unwrap();
        let event = item_event(first_profile.clone());

        assert_eq!(repo.accept_event(&event).unwrap(), AcceptResult::Inserted);
        assert_eq!(repo.accept_event(&event).unwrap(), AcceptResult::Duplicate);
        assert!(repo.events(&second_profile).unwrap().is_empty());
    }

    #[test]
    fn notes_are_profile_scoped_and_reject_overly_long_text() {
        let mut repo = fixture_repo();
        let first = ProfileId::new("1849811906").unwrap();
        let second = ProfileId::new("249165455").unwrap();
        let entity = crate::domain::EntityId::Item(ItemId::new("paper_pondshell").unwrap());
        repo.activate_profile(&first).unwrap();
        repo.activate_profile(&second).unwrap();

        repo.save_note(&first, &entity, "Pond route").unwrap();
        assert_eq!(
            repo.note(&first, &entity).unwrap(),
            Some("Pond route".to_owned())
        );
        assert_eq!(repo.note(&second, &entity).unwrap(), None);
        assert!(matches!(
            repo.save_note(&first, &entity, &"a".repeat(20_001)),
            Err(RepoError::NoteTooLong)
        ));
    }

    #[test]
    fn approved_backup_imports_are_idempotent_and_activate_their_profile() {
        let mut repo = fixture_repo();
        let profile = ProfileId::new("1849811906").unwrap();
        let items = vec![
            ItemId::new("paper_pondshell").unwrap(),
            ItemId::new("test_ore").unwrap(),
        ];

        assert_eq!(
            repo.import_backup_items([7; 32], &profile, "1.0.4", &items)
                .unwrap(),
            ImportResult::Inserted
        );
        assert_eq!(
            repo.import_backup_items([7; 32], &profile, "1.0.4", &items)
                .unwrap(),
            ImportResult::Duplicate
        );
        assert_eq!(repo.active_profile().unwrap(), Some(profile.clone()));
        assert_eq!(repo.events(&profile).unwrap().len(), 2);
    }

    #[test]
    fn replacing_current_save_progress_preserves_notes_and_other_profiles() {
        let mut repo = fixture_repo();
        let reloaded = ProfileId::new("1849811906").unwrap();
        let other = ProfileId::new("249165455").unwrap();
        let note_entity = crate::domain::EntityId::Item(ItemId::new("paper_pondshell").unwrap());
        repo.accept_event(&item_event(reloaded.clone())).unwrap();
        let mut other_event = item_event(other.clone());
        other_event.session_id = Uuid::from_u128(2);
        repo.accept_event(&other_event).unwrap();
        repo.save_note(&reloaded, &note_entity, "Keep this note")
            .unwrap();

        repo.replace_save_state(
            [9; 32],
            &reloaded,
            "1.0.5",
            &[ItemId::new("test_seed").unwrap()],
            r#"{"name":"Ari","farm":"Test Farm"}"#,
        )
        .unwrap();

        let reloaded_events = repo.events(&reloaded).unwrap();
        assert_eq!(reloaded_events.len(), 1);
        assert!(matches!(
            &reloaded_events[0].event,
            CompanionEvent::ItemObtained { item_id, .. } if item_id.as_str() == "test_seed"
        ));
        assert_eq!(repo.events(&other).unwrap().len(), 1);
        assert_eq!(
            repo.note(&reloaded, &note_entity).unwrap().as_deref(),
            Some("Keep this note")
        );
        assert_eq!(
            repo.setting("journal_evidence_v1:1849811906")
                .unwrap()
                .as_deref(),
            Some(r#"{"name":"Ari","farm":"Test Farm"}"#)
        );
        assert_eq!(repo.active_profile().unwrap(), Some(reloaded));
    }

    fn fixture_repo() -> Repository {
        Repository::in_memory().unwrap()
    }

    fn item_event(profile_id: ProfileId) -> EventEnvelope {
        EventEnvelope {
            schema_version: 1,
            companion_version: "0.1.0".to_owned(),
            game_version: "synthetic-1".to_owned(),
            profile_id,
            session_id: Uuid::nil(),
            sequence: 1,
            save_file: None,
            event: CompanionEvent::ItemObtained {
                item_id: ItemId::new("paper_pondshell").unwrap(),
                count: 1,
            },
        }
    }
}
