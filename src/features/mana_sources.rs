use crate::core::models::Decklist;
use crate::ingest::card_database::CardDatabase;
use std::collections::BTreeMap;

pub const COLORS: &[&str] = &["W", "U", "B", "R", "G"];

pub fn source_counts(deck: &Decklist, card_db: &CardDatabase) -> BTreeMap<String, u32> {
    let mut counts = BTreeMap::new();
    for color in COLORS {
        counts.insert((*color).to_string(), 0);
    }
    for (name, count) in &deck.mainboard {
        let Some(card) = card_db.get(name) else {
            continue;
        };
        if !card.is_land() {
            continue;
        }
        for color in card.produces_colors_guess() {
            if COLORS.contains(&color.as_str()) {
                *counts.entry(color).or_insert(0) += count;
            }
        }
    }
    counts
}
