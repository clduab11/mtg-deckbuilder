use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Card {
    pub name: String,
    pub mana_cost: String,
    pub mana_value: f64,
    pub colors: Vec<String>,
    pub color_identity: Vec<String>,
    pub type_line: String,
    pub oracle_text: String,
    pub keywords: Vec<String>,
    pub rarity: String,
    pub set_code: String,
    pub collector_number: String,
    pub legalities: BTreeMap<String, String>,
    pub games: Vec<String>,
    pub arena_id: Option<i64>,
    pub is_basic_land: bool,
    pub is_digital: bool,
    pub is_rebalanced: bool,
}

impl Card {
    pub fn is_land(&self) -> bool {
        self.type_line.contains("Land")
    }

    pub fn produces_colors_guess(&self) -> Vec<String> {
        match self.name.as_str() {
            "Plains" => vec!["W".to_string()],
            "Island" => vec!["U".to_string()],
            "Swamp" => vec!["B".to_string()],
            "Mountain" => vec!["R".to_string()],
            "Forest" => vec!["G".to_string()],
            _ => self.color_identity.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Decklist {
    pub mainboard: IndexMap<String, u32>,
    pub sideboard: IndexMap<String, u32>,
    pub companion: Option<String>,
    pub source_path: Option<String>,
}

impl Decklist {
    pub fn main_count(&self) -> u32 {
        self.mainboard.values().sum()
    }

    pub fn sideboard_count(&self) -> u32 {
        self.sideboard.values().sum()
    }

    pub fn all_counts(&self) -> BTreeMap<String, u32> {
        let mut counts = BTreeMap::new();
        for (name, count) in &self.mainboard {
            *counts.entry(name.clone()).or_insert(0) += *count;
        }
        for (name, count) in &self.sideboard {
            *counts.entry(name.clone()).or_insert(0) += *count;
        }
        if let Some(companion) = &self.companion {
            *counts.entry(companion.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn expanded_mainboard(&self) -> Vec<String> {
        let mut cards = Vec::with_capacity(self.main_count() as usize);
        for (name, count) in &self.mainboard {
            for _ in 0..*count {
                cards.push(name.clone());
            }
        }
        cards
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionIndex {
    pub counts: BTreeMap<String, u32>,
    pub display_names: BTreeMap<String, String>,
    pub source_path: Option<String>,
    pub name_field: Option<String>,
    pub count_field: Option<String>,
    pub warnings: Vec<String>,
}

impl CollectionIndex {
    pub fn owned(&self, name: &str) -> u32 {
        let key = crate::core::normalization::normalize_name(name);
        self.counts.get(&key).copied().unwrap_or(0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationIssue {
    pub card_name: Option<String>,
    pub code: String,
    pub details: BTreeMap<String, Value>,
    pub message: String,
    pub severity: String,
}

impl ValidationIssue {
    pub fn new(
        code: impl Into<String>,
        severity: impl Into<String>,
        message: impl Into<String>,
        card_name: Option<String>,
        details: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            card_name,
            code: code.into(),
            details,
            message: message.into(),
            severity: severity.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationReport {
    pub assumptions: Vec<String>,
    pub format_name: String,
    pub issues: Vec<ValidationIssue>,
    pub main_count: u32,
    pub sideboard_count: u32,
    pub status: String,
    pub wildcards_required: BTreeMap<String, u32>,
}

impl ValidationReport {
    pub fn ok(&self) -> bool {
        self.status == "PASS" && !self.issues.iter().any(|issue| issue.severity == "ERROR")
    }
}
