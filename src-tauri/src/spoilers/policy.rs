use crate::{
    catalog::Catalog,
    domain::{GiftReaction, ItemId, Language, NpcId, SpoilerMode},
    tracking::ProfileProgress,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum FactVisibility {
    Hidden,
    Hinted,
    Revealed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Count {
    pub completed: usize,
    pub total: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CollectionsView {
    pub items: Count,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VisibleItem {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub icon_sprite: Option<String>,
    pub seasons: Vec<String>,
    pub locations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AppSnapshot {
    pub collections: CollectionsView,
    pub items: Vec<VisibleItem>,
    pub villagers: Vec<VillagerView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GiftView {
    pub item_id: ItemId,
    pub item_name: String,
    pub reaction: GiftReaction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VillagerView {
    pub npc_id: NpcId,
    gifts: Vec<GiftView>,
}

impl VillagerView {
    pub fn gift(&self, item: &ItemId) -> Option<&GiftView> {
        self.gifts.iter().find(|gift| gift.item_id == *item)
    }
}

impl AppSnapshot {
    pub fn villager(&self, npc: &NpcId) -> VillagerView {
        self.villagers
            .iter()
            .find(|villager| villager.npc_id == *npc)
            .cloned()
            .unwrap_or(VillagerView {
                npc_id: npc.clone(),
                gifts: vec![],
            })
    }
}

pub struct SpoilerPolicy;

impl SpoilerPolicy {
    pub fn snapshot(
        &self,
        progress: &ProfileProgress,
        catalog: &Catalog,
        mode: SpoilerMode,
        language: Language,
    ) -> AppSnapshot {
        let mut items = Vec::new();
        let mut completed = 0;
        let language = match language {
            Language::Eng => "eng",
            Language::Fra => "fra",
        };
        for item in catalog.items() {
            let discovered = progress.item_is_discovered(&item.id);
            if discovered {
                completed += 1;
            }
            if item_visibility(mode, discovered) == FactVisibility::Revealed {
                let text = item.text(language);
                items.push(VisibleItem {
                    id: item.id.clone(),
                    name: text.name.clone(),
                    description: text.description.clone(),
                    icon_sprite: item.icon_sprite.clone(),
                    seasons: item.seasons.clone(),
                    locations: item
                        .locations
                        .iter()
                        .map(|location| location.as_str().to_owned())
                        .collect(),
                });
            }
        }
        let mut villagers = Vec::new();
        for (npc, item, reaction) in progress.gift_pairs() {
            let npc_id = NpcId::new(npc).expect("progress uses validated NPC IDs");
            let item_id = ItemId::new(item).expect("progress uses validated item IDs");
            let Some(item) = catalog.item(&item_id) else {
                continue;
            };
            let gift = GiftView {
                item_id,
                item_name: item.text(language).name.clone(),
                reaction,
            };
            if let Some(villager) = villagers
                .iter_mut()
                .find(|villager: &&mut VillagerView| villager.npc_id == npc_id)
            {
                villager.gifts.push(gift);
            } else {
                villagers.push(VillagerView {
                    npc_id,
                    gifts: vec![gift],
                });
            }
        }
        AppSnapshot {
            collections: CollectionsView {
                items: Count {
                    completed,
                    total: catalog.items().count(),
                },
            },
            items,
            villagers,
        }
    }
}

pub fn item_visibility(mode: SpoilerMode, discovered: bool) -> FactVisibility {
    match (mode, discovered) {
        (SpoilerMode::All, _) | (SpoilerMode::Free, true) => FactVisibility::Revealed,
        (SpoilerMode::Free, false) => FactVisibility::Hidden,
    }
}
