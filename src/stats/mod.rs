use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use statrs::distribution::{ContinuousCDF, Normal};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, PartialEq)]
pub struct WilsonInterval {
    pub confidence: f64,
    pub lower: f64,
    pub estimate: f64,
    pub upper: f64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, PartialEq)]
pub struct RateSummary {
    pub successes: u32,
    pub samples: u32,
    pub rate: f64,
    pub interval: WilsonInterval,
    pub reliability: String,
    pub sample_size_warning: Option<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, PartialEq, Eq)]
pub struct GameRecord {
    pub match_id: String,
    pub game_number: u8,
    pub queue: String,
    pub format_name: String,
    pub opponent_archetype: Option<String>,
    pub won: bool,
    pub mulligans: u8,
    pub sideboarded: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, PartialEq)]
pub struct ConstructedSummary {
    pub games: u32,
    pub matches: u32,
    pub game_win_rate: RateSummary,
    pub match_win_rate: RateSummary,
    pub bo1_performance: Option<RateSummary>,
    pub bo3_game_performance: Option<RateSummary>,
    pub mulligan_sensitivity: BTreeMap<String, RateSummary>,
    pub matchup_matrix: BTreeMap<String, RateSummary>,
    pub sideboard_impact: Option<RateSummary>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, PartialEq)]
pub struct DraftPickRecord {
    pub draft_id: String,
    pub card_name: String,
    pub pack_number: u8,
    pub pick_number: u8,
    pub seen_at_pick: u8,
    pub taken: bool,
    pub opening_hand_games: u32,
    pub opening_hand_wins: u32,
    pub drawn_games: u32,
    pub drawn_wins: u32,
    pub games: u32,
    pub wins: u32,
    pub trophies: u32,
    pub events: u32,
    pub color_pair: Option<String>,
    pub archetype: Option<String>,
    pub wheeled: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, PartialEq)]
pub struct DraftCardSummary {
    pub card_name: String,
    pub card_win_rate: RateSummary,
    pub game_in_hand_win_rate: RateSummary,
    pub opening_hand_win_rate: RateSummary,
    pub improvement_when_drawn: f64,
    pub average_last_seen_at: f64,
    pub average_taken_at: Option<f64>,
    pub pick_order_score: f64,
    pub trophy_rate: RateSummary,
    pub wheel_rate: RateSummary,
    pub sample_size_reliability: String,
}

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

pub fn rate(successes: u32, samples: u32) -> RateSummary {
    let estimate = if samples == 0 {
        0.0
    } else {
        f64::from(successes) / f64::from(samples)
    };
    let interval = wilson_interval(successes, samples, 0.95);
    let reliability = match samples {
        0..=29 => "low",
        30..=99 => "medium",
        _ => "high",
    }
    .to_string();
    RateSummary {
        successes,
        samples,
        rate: round4(estimate),
        interval,
        reliability,
        sample_size_warning: (samples < 30)
            .then(|| "Fewer than 30 samples; treat this estimate as directional.".to_string()),
    }
}

pub fn wilson_interval(successes: u32, samples: u32, confidence: f64) -> WilsonInterval {
    if samples == 0 {
        return WilsonInterval {
            confidence,
            lower: 0.0,
            estimate: 0.0,
            upper: 0.0,
        };
    }
    let normal = Normal::new(0.0, 1.0).expect("standard normal parameters are valid");
    let z = normal.inverse_cdf(1.0 - (1.0 - confidence) / 2.0);
    let n = f64::from(samples);
    let p = f64::from(successes) / n;
    let denom = 1.0 + z.powi(2) / n;
    let centre = (p + z.powi(2) / (2.0 * n)) / denom;
    let margin = z * ((p * (1.0 - p) / n + z.powi(2) / (4.0 * n.powi(2))).sqrt()) / denom;
    WilsonInterval {
        confidence,
        lower: round4((centre - margin).max(0.0)),
        estimate: round4(p),
        upper: round4((centre + margin).min(1.0)),
    }
}

pub fn summarize_constructed(records: &[GameRecord]) -> ConstructedSummary {
    let game_wins = records.iter().filter(|record| record.won).count() as u32;
    let mut match_ids = BTreeSet::new();
    let mut match_wins = 0;
    for record in records {
        if match_ids.insert(record.match_id.clone()) {
            let match_games: Vec<_> = records
                .iter()
                .filter(|candidate| candidate.match_id == record.match_id)
                .collect();
            let wins = match_games.iter().filter(|game| game.won).count();
            if wins * 2 > match_games.len() {
                match_wins += 1;
            }
        }
    }
    let bo1: Vec<_> = records
        .iter()
        .filter(|record| record.queue == "bo1")
        .collect();
    let bo3: Vec<_> = records
        .iter()
        .filter(|record| record.queue == "bo3")
        .collect();
    let mut mulligan_sensitivity = BTreeMap::new();
    for mulligans in 0..=3 {
        let group: Vec<_> = records
            .iter()
            .filter(|record| record.mulligans == mulligans)
            .collect();
        if !group.is_empty() {
            mulligan_sensitivity.insert(
                mulligans.to_string(),
                rate(
                    group.iter().filter(|record| record.won).count() as u32,
                    group.len() as u32,
                ),
            );
        }
    }
    let mut matchup_matrix = BTreeMap::new();
    for archetype in records
        .iter()
        .filter_map(|record| record.opponent_archetype.clone())
        .collect::<BTreeSet<_>>()
    {
        let group: Vec<_> = records
            .iter()
            .filter(|record| record.opponent_archetype.as_deref() == Some(archetype.as_str()))
            .collect();
        matchup_matrix.insert(
            archetype,
            rate(
                group.iter().filter(|record| record.won).count() as u32,
                group.len() as u32,
            ),
        );
    }
    let sideboarded: Vec<_> = records.iter().filter(|record| record.sideboarded).collect();
    ConstructedSummary {
        games: records.len() as u32,
        matches: match_ids.len() as u32,
        game_win_rate: rate(game_wins, records.len() as u32),
        match_win_rate: rate(match_wins, match_ids.len() as u32),
        bo1_performance: (!bo1.is_empty()).then(|| {
            rate(
                bo1.iter().filter(|record| record.won).count() as u32,
                bo1.len() as u32,
            )
        }),
        bo3_game_performance: (!bo3.is_empty()).then(|| {
            rate(
                bo3.iter().filter(|record| record.won).count() as u32,
                bo3.len() as u32,
            )
        }),
        mulligan_sensitivity,
        matchup_matrix,
        sideboard_impact: (!sideboarded.is_empty()).then(|| {
            rate(
                sideboarded.iter().filter(|record| record.won).count() as u32,
                sideboarded.len() as u32,
            )
        }),
    }
}

pub fn summarize_draft_card(card_name: &str, records: &[DraftPickRecord]) -> DraftCardSummary {
    let card_records: Vec<_> = records
        .iter()
        .filter(|record| record.card_name == card_name)
        .collect();
    let games = card_records.iter().map(|record| record.games).sum::<u32>();
    let wins = card_records.iter().map(|record| record.wins).sum::<u32>();
    let drawn_games = card_records
        .iter()
        .map(|record| record.drawn_games)
        .sum::<u32>();
    let drawn_wins = card_records
        .iter()
        .map(|record| record.drawn_wins)
        .sum::<u32>();
    let opening_games = card_records
        .iter()
        .map(|record| record.opening_hand_games)
        .sum::<u32>();
    let opening_wins = card_records
        .iter()
        .map(|record| record.opening_hand_wins)
        .sum::<u32>();
    let events = card_records.iter().map(|record| record.events).sum::<u32>();
    let trophies = card_records
        .iter()
        .map(|record| record.trophies)
        .sum::<u32>();
    let average_last_seen_at = if card_records.is_empty() {
        0.0
    } else {
        card_records
            .iter()
            .map(|record| f64::from(record.seen_at_pick))
            .sum::<f64>()
            / card_records.len() as f64
    };
    let taken: Vec<_> = card_records.iter().filter(|record| record.taken).collect();
    let average_taken_at = (!taken.is_empty()).then(|| {
        taken
            .iter()
            .map(|record| f64::from(record.pick_number))
            .sum::<f64>()
            / taken.len() as f64
    });
    let base_rate = rate(wins, games);
    let gih = rate(drawn_wins, drawn_games);
    DraftCardSummary {
        card_name: card_name.to_string(),
        improvement_when_drawn: round4(gih.rate - base_rate.rate),
        pick_order_score: round4(1.0 / average_last_seen_at.max(1.0)),
        card_win_rate: base_rate,
        game_in_hand_win_rate: gih,
        opening_hand_win_rate: rate(opening_wins, opening_games),
        average_last_seen_at: round4(average_last_seen_at),
        average_taken_at: average_taken_at.map(round4),
        trophy_rate: rate(trophies, events),
        wheel_rate: rate(
            card_records.iter().filter(|record| record.wheeled).count() as u32,
            card_records.len() as u32,
        ),
        sample_size_reliability: if games < 30 { "low" } else { "medium_or_high" }.to_string(),
    }
}
