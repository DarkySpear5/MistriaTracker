pub mod policy;
pub mod search;

pub use policy::{AppSnapshot, SpoilerPolicy};
pub use search::SearchService;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Language, SpoilerMode};
    use crate::{
        catalog::localization::LocalizedItemText,
        catalog::{Catalog, CatalogItem, CatalogVersion},
        domain::ItemId,
        tracking::ProfileProgress,
    };

    fn fixture_policy() -> SpoilerPolicy {
        SpoilerPolicy
    }

    fn progress_with_one_of_two_items() -> ProfileProgress {
        ProfileProgress::from_discovered([ItemId::new("paper_pondshell").unwrap()])
    }

    fn two_item_catalog() -> Catalog {
        Catalog::new(
            CatalogVersion {
                source_game_version: "synthetic-1".to_owned(),
            },
            vec![
                fixture_item("paper_pondshell", "Paper Pondshell"),
                fixture_item("secret_item", "Secret Item"),
            ],
        )
    }

    fn fixture_item(id: &str, name: &str) -> CatalogItem {
        let text = LocalizedItemText {
            name: name.to_owned(),
            description: format!("Description for {name}"),
        };
        CatalogItem::new(
            ItemId::new(id).unwrap(),
            None,
            vec![],
            vec![],
            text.clone(),
            text,
        )
    }

    #[test]
    fn spoiler_free_hides_identity_but_preserves_aggregate_total() {
        let view = fixture_policy().snapshot(
            &progress_with_one_of_two_items(),
            &two_item_catalog(),
            SpoilerMode::Free,
            Language::Fra,
        );
        assert_eq!(view.collections.items.completed, 1);
        assert_eq!(view.collections.items.total, 2);
        assert_eq!(
            view.items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["paper_pondshell"]
        );
        assert!(SearchService::search(&view, "secret").is_empty());
    }

    #[test]
    fn gift_pair_reveals_only_after_that_pair_is_tried() {
        let adeline = crate::domain::NpcId::new("adeline").unwrap();
        let balor = crate::domain::NpcId::new("balor").unwrap();
        let item = ItemId::new("paper_pondshell").unwrap();
        let progress = ProfileProgress::from_gift_pair(
            adeline.clone(),
            item.clone(),
            crate::domain::GiftReaction::Liked,
        );

        let view = fixture_policy().snapshot(
            &progress,
            &two_item_catalog(),
            SpoilerMode::Free,
            Language::Fra,
        );

        assert_eq!(
            view.villager(&adeline).gift(&item).unwrap().reaction,
            crate::domain::GiftReaction::Liked
        );
        assert_eq!(
            view.villager(&adeline).gift(&item).unwrap().item_name,
            "Paper Pondshell"
        );
        assert!(view.villager(&balor).gift(&item).is_none());
    }

    #[test]
    fn snapshot_serializes_only_spoiler_safe_fields_for_the_desktop_boundary() {
        let view = fixture_policy().snapshot(
            &progress_with_one_of_two_items(),
            &two_item_catalog(),
            SpoilerMode::Free,
            Language::Eng,
        );

        assert_eq!(
            serde_json::to_value(view).unwrap(),
            serde_json::json!({
                "collections": {"items": {"completed": 1, "total": 2}},
                "items": [{
                    "id": "paper_pondshell",
                    "name": "Paper Pondshell",
                    "description": "Description for Paper Pondshell",
                    "icon_sprite": null,
                    "seasons": [],
                    "locations": []
                }],
                "villagers": []
            })
        );
    }
}
