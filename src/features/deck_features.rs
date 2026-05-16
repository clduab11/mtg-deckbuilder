use crate::core::models::Decklist;
use crate::features::card_roles::classify_card_roles;
use crate::features::mana_curve::mana_curve;
use crate::features::mana_sources::source_counts;
use crate::ingest::card_database::CardDatabase;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

pub fn extract_deck_features(deck: &Decklist, card_db: &CardDatabase) -> Value {
    let mut land_count = 0;
    let mut nonland_count = 0;
    let mut nonland_mv_total = 0.0;
    let mut role_counts: BTreeMap<String, u32> = BTreeMap::new();

    for (name, count) in &deck.mainboard {
        let Some(card) = card_db.get(name) else {
            continue;
        };
        if card.is_land() {
            land_count += count;
        } else {
            nonland_count += count;
            nonland_mv_total += card.mana_value * f64::from(*count);
        }
        for role in classify_card_roles(card) {
            *role_counts.entry(role).or_insert(0) += count;
        }
    }

    let deck_size = deck.main_count().max(1);
    let threat_count = role_counts.get("threat").copied().unwrap_or(0);
    let interaction_count = role_counts.get("interaction").copied().unwrap_or(0);
    let protection_count = role_counts.get("protection").copied().unwrap_or(0);
    let average_mv = if nonland_count == 0 {
        0.0
    } else {
        nonland_mv_total / f64::from(nonland_count)
    };

    json!({
        "average_nonland_mana_value": round3(average_mv),
        "deck_size": deck.main_count(),
        "interaction_density": f64::from(interaction_count) / f64::from(deck_size),
        "land_count": land_count,
        "land_ratio": f64::from(land_count) / f64::from(deck_size),
        "mana_curve": mana_curve(deck, card_db, 7),
        "nonland_count": nonland_count,
        "protection_density": f64::from(protection_count) / f64::from(deck_size),
        "role_counts": role_counts,
        "source_counts": source_counts(deck, card_db),
        "threat_density": f64::from(threat_count) / f64::from(deck_size),
    })
}
