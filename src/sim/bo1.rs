use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Bo1Config {
    pub queue: String,
    pub opening_hand_mode: String,
    pub max_mulligans: u32,
    pub sideboard_slots_available: u8,
    pub games_per_match: u8,
    pub assumptions: Vec<String>,
}

impl Default for Bo1Config {
    fn default() -> Self {
        Self {
            queue: "bo1-oriented".to_string(),
            opening_hand_mode: "arena_n2".to_string(),
            max_mulligans: 2,
            sideboard_slots_available: 7,
            games_per_match: 1,
            assumptions: vec![
                "Bo1-oriented configuration for offline analysis; not exact MTG Arena parity."
                    .to_string(),
                "Arena-like opening hand smoothing is an approximation because the exact algorithm is not public."
                    .to_string(),
                "Constructed Bo1 sideboard modeling assumes seven accessible cards for wishboard-style effects."
                    .to_string(),
            ],
        }
    }
}
