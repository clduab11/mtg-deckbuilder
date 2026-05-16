use crate::core::models::Card;
use std::collections::BTreeSet;

pub fn classify_card_roles(card: &Card) -> BTreeSet<String> {
    let text = format!("{}\n{}", card.type_line, card.oracle_text).to_lowercase();
    let mut roles = BTreeSet::new();
    if card.is_land() {
        roles.insert("land".to_string());
    }
    if text.contains("creature") || text.contains("planeswalker") || text.contains("battle") {
        roles.insert("threat".to_string());
    }
    if [
        "destroy target",
        "exile target",
        "deals",
        "damage to any target",
        "counter target",
    ]
    .iter()
    .any(|term| text.contains(term))
    {
        roles.insert("interaction".to_string());
    }
    if [
        "draw a card",
        "draw two",
        "look at the top",
        "surveil",
        "scry",
    ]
    .iter()
    .any(|term| text.contains(term))
    {
        roles.insert("selection_or_card_advantage".to_string());
    }
    if ["hexproof", "indestructible", "protection", "ward"]
        .iter()
        .any(|term| text.contains(term))
    {
        roles.insert("protection".to_string());
    }
    if text.contains("landfall") || text.contains("whenever a land enters") {
        roles.insert("landfall".to_string());
    }
    roles
}
