use crate::export::arena::format_arena_decklist;
use crate::ingest::card_database::CardDatabase;
use crate::ingest::decklist_importer::parse_arena_decklist_text;
use crate::rules::validator::DeckValidator;
use crate::sim::opening_hand::simulate_opening_hands;
use anyhow::Result;
use serde_json::Value;

pub fn validate_deck(deck_text: &str, card_data_path: &str, format_name: &str) -> Result<Value> {
    let db = CardDatabase::from_scryfall_file(card_data_path)?;
    let deck = parse_arena_decklist_text(deck_text)?;
    Ok(serde_json::to_value(DeckValidator::new(db).validate(
        &deck,
        format_name,
        None,
        false,
        None,
    ))?)
}

pub fn export_arena_decklist(deck_text: &str) -> Result<String> {
    Ok(format_arena_decklist(&parse_arena_decklist_text(
        deck_text,
    )?))
}

pub fn simulate_opening_hands_tool(
    deck_text: &str,
    card_data_path: &str,
    trials: u32,
    mode: &str,
) -> Result<Value> {
    let db = CardDatabase::from_scryfall_file(card_data_path)?;
    let deck = parse_arena_decklist_text(deck_text)?;
    Ok(serde_json::to_value(simulate_opening_hands(
        &deck, &db, mode, trials, 1, 2,
    )?)?)
}
