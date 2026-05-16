use crate::core::models::{CollectionIndex, Decklist};
use crate::ingest::card_database::CardDatabase;
use std::collections::BTreeMap;

pub fn compute_wildcards_required(
    deck: &Decklist,
    card_db: &CardDatabase,
    collection: &CollectionIndex,
) -> BTreeMap<String, u32> {
    let mut needed = BTreeMap::new();
    for (name, count) in deck.all_counts() {
        let Some(card) = card_db.get(&name) else {
            continue;
        };
        if card.is_basic_land {
            continue;
        }
        let missing = count.saturating_sub(collection.owned(&name));
        if missing > 0 {
            let mut rarity = card.rarity.to_lowercase();
            if !["common", "uncommon", "rare", "mythic"].contains(&rarity.as_str()) {
                rarity = "common".to_string();
            }
            *needed.entry(rarity).or_insert(0) += missing;
        }
    }
    needed
}
