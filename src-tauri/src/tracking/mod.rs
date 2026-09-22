pub mod log_lines;
pub mod log_tail;
pub mod session;

pub use log_tail::{LogTailError, ReadOnlyLogTail};
pub use session::PassiveTrackingSession;

use crate::domain::{CompanionEvent, EventEnvelope, GiftReaction, ItemId, NpcId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub struct ProfileProgress {
    discovered_items: BTreeSet<String>,
    donated_items: BTreeSet<String>,
    gift_pairs: BTreeMap<(String, String), GiftReaction>,
}

impl ProfileProgress {
    pub fn from_events(events: impl IntoIterator<Item = EventEnvelope>) -> Self {
        let mut progress = Self::default();
        for event in events {
            match event.event {
                CompanionEvent::MuseumDonated { item_id, .. } => {
                    progress.donated_items.insert(item_id.as_str().to_owned());
                    progress
                        .discovered_items
                        .insert(item_id.as_str().to_owned());
                }
                CompanionEvent::ItemObtained { item_id, .. } => {
                    progress
                        .discovered_items
                        .insert(item_id.as_str().to_owned());
                }
                CompanionEvent::GiftGiven {
                    npc_id,
                    item_id,
                    reaction,
                } => {
                    progress.gift_pairs.insert(
                        (npc_id.as_str().to_owned(), item_id.as_str().to_owned()),
                        reaction,
                    );
                }
                CompanionEvent::ProfileActivated | CompanionEvent::ProfileDeactivated => {}
            }
        }
        progress
    }

    pub fn from_discovered(items: impl IntoIterator<Item = ItemId>) -> Self {
        Self {
            discovered_items: items
                .into_iter()
                .map(|item| item.as_str().to_owned())
                .collect(),
            gift_pairs: BTreeMap::new(),
            donated_items: BTreeSet::new(),
        }
    }

    pub fn item_is_discovered(&self, item: &ItemId) -> bool {
        self.discovered_items.contains(item.as_str())
    }

    pub fn donated_items(&self) -> impl Iterator<Item = &String> {
        self.donated_items.iter()
    }

    pub fn from_gift_pair(npc: NpcId, item: ItemId, reaction: GiftReaction) -> Self {
        let mut progress = Self::default();
        progress.gift_pairs.insert(
            (npc.as_str().to_owned(), item.as_str().to_owned()),
            reaction,
        );
        progress
    }

    pub fn gift_pairs(&self) -> impl Iterator<Item = (&str, &str, GiftReaction)> {
        self.gift_pairs
            .iter()
            .map(|((npc, item), reaction)| (npc.as_str(), item.as_str(), *reaction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CompanionEvent, EventEnvelope, GiftReaction, ItemId, NpcId, ProfileId};
    use uuid::Uuid;

    #[test]
    fn derives_only_observed_items_and_exact_gift_pairs() {
        let progress = ProfileProgress::from_events([
            event(CompanionEvent::ItemObtained {
                item_id: ItemId::new("paper_pondshell").unwrap(),
                count: 1,
            }),
            event(CompanionEvent::GiftGiven {
                npc_id: NpcId::new("juniper").unwrap(),
                item_id: ItemId::new("paper_pondshell").unwrap(),
                reaction: GiftReaction::Liked,
            }),
        ]);

        assert!(progress.item_is_discovered(&ItemId::new("paper_pondshell").unwrap()));
        assert_eq!(
            progress.gift_pairs().collect::<Vec<_>>(),
            vec![("juniper", "paper_pondshell", GiftReaction::Liked)]
        );
    }

    fn event(event: CompanionEvent) -> EventEnvelope {
        EventEnvelope {
            schema_version: 1,
            companion_version: "test".to_owned(),
            game_version: "test".to_owned(),
            profile_id: ProfileId::new("1849811906").unwrap(),
            session_id: Uuid::nil(),
            sequence: 1,
            save_file: None,
            event,
        }
    }
}
