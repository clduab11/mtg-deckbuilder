use crate::core::models::Card;
use crate::core::normalization::{is_basic_land_name, normalize_name};
use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct CardDatabase {
    cards_by_key: BTreeMap<String, Card>,
    pub source: Option<String>,
    pub warnings: Vec<String>,
}

impl CardDatabase {
    pub fn new(source: Option<String>) -> Self {
        Self {
            cards_by_key: BTreeMap::new(),
            source,
            warnings: Vec::new(),
        }
    }

    pub fn add(&mut self, card: Card) {
        self.cards_by_key.insert(normalize_name(&card.name), card);
    }

    pub fn get(&self, name: &str) -> Option<&Card> {
        self.cards_by_key.get(&normalize_name(name))
    }

    pub fn require(&self, name: &str) -> Result<&Card> {
        self.get(name)
            .ok_or_else(|| anyhow!("Unknown card in CardDatabase: {name}"))
    }

    pub fn from_cards(cards: impl IntoIterator<Item = Card>, source: Option<String>) -> Self {
        let mut db = Self::new(source);
        for card in cards {
            db.add(card);
        }
        db
    }

    pub fn from_scryfall_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let cards = crate::ingest::scryfall_loader::load_scryfall_cards(path)?;
        Ok(Self::from_cards(
            cards,
            Some(path.to_string_lossy().to_string()),
        ))
    }

    pub fn from_mtgjson_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let cards = crate::ingest::mtgjson_loader::load_mtgjson_cards(path)?;
        Ok(Self::from_cards(
            cards,
            Some(path.to_string_lossy().to_string()),
        ))
    }
}

pub fn basic_land_card(name: &str) -> Card {
    let colors = match name {
        "Plains" => vec!["W".to_string()],
        "Island" => vec!["U".to_string()],
        "Swamp" => vec!["B".to_string()],
        "Mountain" => vec!["R".to_string()],
        "Forest" => vec!["G".to_string()],
        _ => Vec::new(),
    };
    Card {
        name: name.to_string(),
        mana_cost: String::new(),
        mana_value: 0.0,
        colors: Vec::new(),
        color_identity: colors,
        type_line: "Basic Land".to_string(),
        oracle_text: String::new(),
        keywords: Vec::new(),
        rarity: "common".to_string(),
        set_code: String::new(),
        collector_number: String::new(),
        legalities: BTreeMap::new(),
        games: vec!["arena".to_string()],
        arena_id: None,
        is_basic_land: is_basic_land_name(name),
        is_digital: false,
        is_rebalanced: false,
    }
}
