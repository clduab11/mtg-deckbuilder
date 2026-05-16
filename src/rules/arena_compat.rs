use crate::core::models::{Decklist, ValidationIssue};
use std::collections::BTreeMap;

pub fn check_arena_import_compatibility(deck: &Decklist) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    for (zone_name, zone) in [
        ("mainboard", &deck.mainboard),
        ("sideboard", &deck.sideboard),
    ] {
        for (card_name, count) in zone {
            if *count == 0 {
                issues.push(ValidationIssue::new(
                    "arena.count",
                    "ERROR",
                    format!("Nonpositive count in {zone_name}"),
                    Some(card_name.clone()),
                    BTreeMap::new(),
                ));
            }
            if card_name.contains('\t') || card_name.contains('\r') || card_name.contains('\n') {
                issues.push(ValidationIssue::new(
                    "arena.name",
                    "ERROR",
                    format!("Invalid control character in {zone_name} name"),
                    Some(card_name.clone()),
                    BTreeMap::new(),
                ));
            }
        }
    }
    issues
}
