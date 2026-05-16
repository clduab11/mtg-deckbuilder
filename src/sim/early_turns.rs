use crate::core::models::Decklist;
use crate::ingest::card_database::CardDatabase;
use crate::sim::opening_hand::SeededRng;
use anyhow::{Result, anyhow};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize)]
pub struct EarlyTurnSimulationResult {
    pub assumptions: Vec<String>,
    pub metrics: Value,
    pub seed: u64,
    pub trials: u32,
}

fn is_castable_early(
    card_name: &str,
    available_colors: &BTreeSet<String>,
    available_mana: u32,
    card_db: &CardDatabase,
) -> bool {
    let Some(card) = card_db.get(card_name) else {
        return false;
    };
    if card.is_land() || card.mana_value > f64::from(available_mana) {
        return false;
    }
    card.colors.is_empty()
        || card
            .colors
            .iter()
            .any(|color| available_colors.contains(color))
}

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

pub fn simulate_first_three_turns(
    deck: &Decklist,
    card_db: &CardDatabase,
    trials: u32,
    seed: u64,
) -> Result<EarlyTurnSimulationResult> {
    let cards = deck.expanded_mainboard();
    if cards.len() < 10 {
        return Err(anyhow!(
            "Deck must contain enough cards to simulate opening plus three draws"
        ));
    }
    let mut rng = SeededRng::new(seed);
    let mut counters: BTreeMap<String, u32> = BTreeMap::new();

    for _ in 0..trials {
        let draw = rng.sample(&cards, 10);
        let mut hand = draw[..7].to_vec();
        let library_draws = draw[7..].to_vec();
        let mut lands_played = 0;
        let mut available_colors = BTreeSet::new();
        let mut relevant_actions = 0;
        let mut threat_played = false;

        for turn in 1..=3 {
            hand.push(library_draws[turn - 1].clone());
            if let Some(land_idx) = hand
                .iter()
                .position(|name| card_db.get(name).is_some_and(|card| card.is_land()))
            {
                let land_to_play = hand.remove(land_idx);
                lands_played += 1;
                if let Some(card) = card_db.get(&land_to_play) {
                    for color in card.produces_colors_guess() {
                        available_colors.insert(color);
                    }
                }
            }

            let mut chosen_idx = None;
            let mut chosen_mv = f64::MAX;
            for (idx, name) in hand.iter().enumerate() {
                if is_castable_early(name, &available_colors, lands_played, card_db) {
                    let mv = card_db
                        .get(name)
                        .map(|card| card.mana_value)
                        .unwrap_or(99.0);
                    if mv < chosen_mv {
                        chosen_idx = Some(idx);
                        chosen_mv = mv;
                    }
                }
            }

            if let Some(idx) = chosen_idx {
                let chosen = hand.remove(idx);
                relevant_actions += 1;
                if let Some(card) = card_db.get(&chosen)
                    && (card.type_line.contains("Creature")
                        || card.type_line.contains("Planeswalker"))
                {
                    threat_played = true;
                }
                if turn == 2 {
                    *counters
                        .entry("turn_2_plan_online".to_string())
                        .or_insert(0) += 1;
                }
                if turn == 3 {
                    *counters
                        .entry("turn_3_plan_online".to_string())
                        .or_insert(0) += 1;
                }
            }
        }

        *counters.entry("trials".to_string()).or_insert(0) += 1;
        if relevant_actions == 0 {
            *counters
                .entry("did_nothing_by_turn_3".to_string())
                .or_insert(0) += 1;
        }
        if threat_played {
            *counters.entry("threat_by_turn_3".to_string()).or_insert(0) += 1;
        }
        if lands_played < 3 {
            *counters
                .entry("missed_land_drop_before_turn_3".to_string())
                .or_insert(0) += 1;
        }
    }

    let denom = f64::from(counters.get("trials").copied().unwrap_or(1).max(1));
    Ok(EarlyTurnSimulationResult {
        assumptions: vec![
            "Deterministic sequencing heuristic; not a gameplay simulator and not a match win-rate model."
                .to_string(),
        ],
        metrics: json!({
            "did_nothing_by_turn_3_rate": round4(f64::from(counters.get("did_nothing_by_turn_3").copied().unwrap_or(0)) / denom),
            "missed_land_drop_before_turn_3_rate": round4(f64::from(counters.get("missed_land_drop_before_turn_3").copied().unwrap_or(0)) / denom),
            "threat_by_turn_3_rate": round4(f64::from(counters.get("threat_by_turn_3").copied().unwrap_or(0)) / denom),
            "turn_2_plan_online_rate": round4(f64::from(counters.get("turn_2_plan_online").copied().unwrap_or(0)) / denom),
            "turn_3_plan_online_rate": round4(f64::from(counters.get("turn_3_plan_online").copied().unwrap_or(0)) / denom),
        }),
        seed,
        trials,
    })
}
