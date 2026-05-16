use crate::core::models::Decklist;
use crate::features::deck_features::extract_deck_features;
use crate::ingest::card_database::CardDatabase;

pub fn infer_archetype_tags(deck: &Decklist, card_db: &CardDatabase) -> Vec<String> {
    let features = extract_deck_features(deck, card_db);
    let mut tags = Vec::new();
    let role_counts = features["role_counts"].as_object();
    if role_counts
        .and_then(|roles| roles.get("landfall"))
        .and_then(|value| value.as_u64())
        .unwrap_or(0)
        >= 6
    {
        tags.push("landfall".to_string());
    }
    if role_counts
        .and_then(|roles| roles.get("interaction"))
        .and_then(|value| value.as_u64())
        .unwrap_or(0)
        >= 10
    {
        tags.push("interactive".to_string());
    }
    if features["average_nonland_mana_value"]
        .as_f64()
        .unwrap_or(0.0)
        <= 2.5
        && features["threat_density"].as_f64().unwrap_or(0.0) >= 0.25
    {
        tags.push("low-curve".to_string());
    }
    if tags.is_empty() {
        vec!["unclassified".to_string()]
    } else {
        tags
    }
}
