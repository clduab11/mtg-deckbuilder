use crate::core::models::Decklist;
use crate::ingest::card_database::CardDatabase;
use crate::sim::opening_hand::simulate_opening_hands;
use anyhow::Result;
use serde_json::{Value, json};

pub fn run_bad_variance_stress_test(
    deck: &Decklist,
    card_db: &CardDatabase,
    trials: u32,
    seed: u64,
) -> Result<Value> {
    let paper = simulate_opening_hands(deck, card_db, "paper", trials, seed, 2)?;
    Ok(json!({
        "baseline": paper,
        "mode": "bad_variance_smoke",
        "tracked_conditions": [
            "too_few_lands",
            "too_many_lands",
            "no_early_play",
            "wrong_colors",
            "no_threat"
        ]
    }))
}
