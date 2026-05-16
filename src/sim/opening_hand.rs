use crate::core::models::Decklist;
use crate::ingest::card_database::CardDatabase;
use crate::sim::arena_like_smoother::{score_candidate_hand, select_arena_like_hand};
use anyhow::{Result, anyhow};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub struct SeededRng {
    rng: ChaCha8Rng,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    fn randbelow(&mut self, upper: usize) -> usize {
        self.rng.random_range(0..upper)
    }

    pub fn sample<T: Clone>(&mut self, population: &[T], k: usize) -> Vec<T> {
        let n = population.len();
        assert!(k <= n, "sample larger than population");
        let mut pool = population.to_vec();
        let mut result = Vec::with_capacity(k);
        for i in 0..k {
            let j = i + self.randbelow(n - i);
            pool.swap(i, j);
            result.push(pool[i].clone());
        }
        result
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct OpeningHandSimulationResult {
    pub assumptions: Vec<String>,
    pub metrics: Value,
    pub mode: String,
    pub seed: u64,
    pub trials: u32,
}

fn deck_land_count(deck: &Decklist, card_db: &CardDatabase) -> u32 {
    deck.mainboard
        .iter()
        .filter_map(|(name, count)| {
            card_db
                .get(name)
                .filter(|card| card.is_land())
                .map(|_| count)
        })
        .sum()
}

fn hard_keep(hand: &[String], card_db: &CardDatabase) -> (bool, Vec<String>) {
    let mut land_count = 0;
    let mut early_play = 0;
    let mut threat = 0;
    let mut source_symbols = BTreeSet::new();
    let mut spell_symbols = BTreeSet::new();
    for name in hand {
        let Some(card) = card_db.get(name) else {
            continue;
        };
        if card.is_land() {
            land_count += 1;
            for color in card.produces_colors_guess() {
                source_symbols.insert(color);
            }
        } else {
            if card.mana_value <= 2.0 {
                early_play += 1;
            }
            if card.type_line.contains("Creature") || card.type_line.contains("Planeswalker") {
                threat += 1;
            }
            for color in &card.colors {
                spell_symbols.insert(color.clone());
            }
        }
    }
    let mut reasons = Vec::new();
    if land_count == 0 {
        reasons.push("0 lands".to_string());
    }
    if land_count == 1 && early_play == 0 {
        reasons.push("1 land and no early play".to_string());
    }
    if land_count >= 6 {
        reasons.push("6+ lands".to_string());
    }
    if !spell_symbols.is_empty() && source_symbols.intersection(&spell_symbols).next().is_none() {
        reasons.push("wrong colors".to_string());
    }
    if early_play == 0 {
        reasons.push("no early play".to_string());
    }
    if threat == 0 {
        reasons.push("no threat".to_string());
    }
    (reasons.is_empty(), reasons)
}

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

pub fn simulate_opening_hands(
    deck: &Decklist,
    card_db: &CardDatabase,
    mode: &str,
    trials: u32,
    seed: u64,
    max_mulligans: u32,
) -> Result<OpeningHandSimulationResult> {
    let cards = deck.expanded_mainboard();
    if cards.len() < 7 {
        return Err(anyhow!("Deck must contain at least 7 mainboard cards"));
    }
    let n_candidates = match mode {
        "paper" => 1,
        "arena_n2" => 2,
        "arena_n3" => 3,
        _ => return Err(anyhow!("Unsupported opening-hand simulation mode: {mode}")),
    };

    let mut rng = SeededRng::new(seed);
    let land_count_deck = deck_land_count(deck, card_db);
    let mut counters: BTreeMap<String, u32> = BTreeMap::new();

    for _ in 0..trials {
        let mut mulligans = 0;
        let mut final_score = None;
        while mulligans <= max_mulligans {
            let score = if mode == "paper" {
                let hand = rng.sample(&cards, 7);
                score_candidate_hand(&hand, cards.len(), land_count_deck, card_db)
            } else {
                select_arena_like_hand(&cards, land_count_deck, card_db, n_candidates, &mut rng)?
            };
            let (keep, reasons) = hard_keep(&score.hand, card_db);
            if keep || mulligans == max_mulligans {
                if !keep {
                    for reason in reasons {
                        *counters
                            .entry(format!("forced_keep_after_mulligan:{reason}"))
                            .or_insert(0) += 1;
                    }
                }
                final_score = Some(score);
                break;
            }
            mulligans += 1;
        }

        let score = final_score.expect("loop always assigns a score");
        let (keep, reasons) = hard_keep(&score.hand, card_db);
        *counters.entry("hands".to_string()).or_insert(0) += 1;
        *counters
            .entry(format!("mulligan_to_{}", 7_u32.saturating_sub(mulligans)))
            .or_insert(0) += 1;
        if mulligans == 0 && keep {
            *counters.entry("keepable_7".to_string()).or_insert(0) += 1;
        }
        if score.land_count <= 1 {
            *counters
                .entry("screw_risk_opening".to_string())
                .or_insert(0) += 1;
        }
        if score.land_count >= 5 {
            *counters
                .entry("flood_risk_opening".to_string())
                .or_insert(0) += 1;
        }
        if score.color_access_score <= 0.0 {
            *counters.entry("no_primary_source".to_string()).or_insert(0) += 1;
        }
        if score.early_play_score > 0.0 {
            *counters.entry("turn_1_or_2_play".to_string()).or_insert(0) += 1;
        }
        if score.plan_score > 0.0 {
            *counters.entry("has_threat".to_string()).or_insert(0) += 1;
        }
        if !keep {
            *counters
                .entry("low_quality_forced_keep".to_string())
                .or_insert(0) += 1;
        }
        for reason in reasons {
            *counters.entry(format!("issue:{reason}")).or_insert(0) += 1;
        }
    }

    let denom = f64::from(counters.get("hands").copied().unwrap_or(1).max(1));
    let metrics = json!({
        "flood_risk_opening_rate": round4(f64::from(counters.get("flood_risk_opening").copied().unwrap_or(0)) / denom),
        "has_threat_rate": round4(f64::from(counters.get("has_threat").copied().unwrap_or(0)) / denom),
        "keepable_7_rate": round4(f64::from(counters.get("keepable_7").copied().unwrap_or(0)) / denom),
        "low_quality_forced_keep_rate": round4(f64::from(counters.get("low_quality_forced_keep").copied().unwrap_or(0)) / denom),
        "mulligan_to_5_rate": round4(f64::from(counters.get("mulligan_to_5").copied().unwrap_or(0)) / denom),
        "mulligan_to_6_rate": round4(f64::from(counters.get("mulligan_to_6").copied().unwrap_or(0)) / denom),
        "no_primary_source_rate": round4(f64::from(counters.get("no_primary_source").copied().unwrap_or(0)) / denom),
        "screw_risk_opening_rate": round4(f64::from(counters.get("screw_risk_opening").copied().unwrap_or(0)) / denom),
        "turn_1_or_2_play_rate": round4(f64::from(counters.get("turn_1_or_2_play").copied().unwrap_or(0)) / denom),
    });
    let mut assumptions = vec!["Opening-hand quality is not match win rate.".to_string()];
    if mode.starts_with("arena") {
        assumptions.push(
            "Arena-like approximation; exact MTG Arena Bo1 hand smoothing is not public."
                .to_string(),
        );
    } else {
        assumptions
            .push("Paper-random 7-card sampling with London-style mulligan heuristic.".to_string());
    }

    Ok(OpeningHandSimulationResult {
        assumptions,
        metrics,
        mode: mode.to_string(),
        seed,
        trials,
    })
}
