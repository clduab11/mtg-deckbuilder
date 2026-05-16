pub use crate::core::models::{Card, CollectionIndex, Decklist, ValidationIssue, ValidationReport};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Queue {
    Bo1,
    Bo3,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MatchConfig {
    pub queue: Queue,
    pub format_name: String,
    pub seed: u64,
    pub trials: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ArchetypeTag {
    pub name: String,
    pub colors: Vec<String>,
    pub strategy: String,
}
