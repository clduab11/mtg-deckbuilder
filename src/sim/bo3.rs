use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Bo3Config {
    pub queue: String,
    pub opening_hand_mode: String,
    pub max_mulligans: u32,
    pub sideboard_slots_available: u8,
    pub games_per_match: u8,
    pub assumptions: Vec<String>,
}

impl Default for Bo3Config {
    fn default() -> Self {
        Self {
            queue: "bo3-oriented".to_string(),
            opening_hand_mode: "paper".to_string(),
            max_mulligans: 2,
            sideboard_slots_available: 15,
            games_per_match: 3,
            assumptions: vec![
                "Bo3-oriented configuration for offline analysis; not exact MTG Arena parity."
                    .to_string(),
                "Sideboard impact is modeled from supplied result data, not from live opponent inference."
                    .to_string(),
                "Match-level results are aggregates of deterministic game records and seeded simulation proxies."
                    .to_string(),
            ],
        }
    }
}
