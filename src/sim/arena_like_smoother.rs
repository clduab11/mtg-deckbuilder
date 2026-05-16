use crate::ingest::card_database::CardDatabase;
use crate::sim::opening_hand::SeededRng;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct CandidateHandScore {
    pub hand: Vec<String>,
    pub land_count: u32,
    pub land_delta: f64,
    pub keep_score: f64,
    pub color_access_score: f64,
    pub early_play_score: f64,
    pub plan_score: f64,
}

impl CandidateHandScore {
    fn ranking_key(&self) -> (f64, f64, f64, f64) {
        (
            self.land_delta,
            -self.keep_score,
            -self.color_access_score,
            -self.early_play_score,
        )
    }

    fn is_less_than(&self, other: &Self) -> bool {
        let a = self.ranking_key();
        let b = other.ranking_key();
        a.0 < b.0
            || (a.0 == b.0
                && (a.1 < b.1 || (a.1 == b.1 && (a.2 < b.2 || (a.2 == b.2 && a.3 < b.3)))))
    }
}

pub fn score_candidate_hand(
    hand: &[String],
    deck_size: usize,
    deck_land_count: u32,
    card_db: &CardDatabase,
) -> CandidateHandScore {
    let expected_lands = 7.0 * f64::from(deck_land_count) / deck_size.max(1) as f64;
    let land_count = hand
        .iter()
        .filter(|name| card_db.get(name).is_some_and(|card| card.is_land()))
        .count() as u32;
    let land_delta = (f64::from(land_count) - expected_lands).abs();
    let mut cards: BTreeMap<&str, u32> = BTreeMap::new();
    for name in hand {
        *cards.entry(name.as_str()).or_insert(0) += 1;
    }

    let mut early_play_count = 0;
    let mut threat_count = 0;
    let mut color_symbols = std::collections::BTreeSet::new();
    let mut source_symbols = std::collections::BTreeSet::new();

    for (name, count) in cards {
        let Some(card) = card_db.get(name) else {
            continue;
        };
        if card.is_land() {
            for color in card.produces_colors_guess() {
                source_symbols.insert(color);
            }
        } else {
            if card.mana_value <= 2.0 {
                early_play_count += count;
            }
            if card.type_line.contains("Creature") || card.type_line.contains("Planeswalker") {
                threat_count += count;
            }
            for color in &card.colors {
                color_symbols.insert(color.clone());
            }
        }
    }

    let intersection = source_symbols.intersection(&color_symbols).count();
    let color_access_score = intersection as f64 / color_symbols.len().max(1) as f64;
    let early_play_score = (f64::from(early_play_count) / 2.0).min(1.0);
    let plan_score = (f64::from(threat_count) / 2.0).min(1.0);
    let mut keep_score = 0.0;
    if (2..=4).contains(&land_count) {
        keep_score += 0.45;
    } else if land_count == 1 || land_count == 5 {
        keep_score += 0.15;
    }
    keep_score += 0.25 * color_access_score;
    keep_score += 0.20 * early_play_score;
    keep_score += 0.10 * plan_score;

    CandidateHandScore {
        hand: hand.to_vec(),
        land_count,
        land_delta,
        keep_score,
        color_access_score,
        early_play_score,
        plan_score,
    }
}

pub fn select_arena_like_hand(
    deck_cards: &[String],
    deck_land_count: u32,
    card_db: &CardDatabase,
    n: usize,
    rng: &mut SeededRng,
) -> anyhow::Result<CandidateHandScore> {
    if n < 1 {
        anyhow::bail!("n must be >= 1");
    }
    if deck_cards.len() < 7 {
        anyhow::bail!("deck must contain at least 7 cards");
    }
    let mut best = None;
    for _ in 0..n {
        let hand = rng.sample(deck_cards, 7);
        let score = score_candidate_hand(&hand, deck_cards.len(), deck_land_count, card_db);
        if best
            .as_ref()
            .is_none_or(|current: &CandidateHandScore| score.is_less_than(current))
        {
            best = Some(score);
        }
    }
    Ok(best.expect("at least one candidate"))
}
