use crate::core::models::Card;
use crate::core::normalization::is_basic_land_name;
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

fn as_vec(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Some(Value::String(item)) => vec![item.clone()],
        _ => Vec::new(),
    }
}

fn first_face_text(card: &Value, key: &str) -> String {
    if let Some(value) = card.get(key)
        && !value.is_null()
    {
        return value
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| value.to_string());
    }
    card.get("card_faces")
        .and_then(Value::as_array)
        .map(|faces| {
            faces
                .iter()
                .filter_map(|face| face.get(key).and_then(Value::as_str))
                .filter(|part| !part.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
                .join(" // ")
        })
        .unwrap_or_default()
}

pub fn load_scryfall_cards(path: impl AsRef<Path>) -> Result<Vec<Card>> {
    let data: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let records = if let Some(records) = data.get("data").and_then(Value::as_array) {
        records.clone()
    } else if let Some(records) = data.as_array() {
        records.clone()
    } else {
        return Err(anyhow!(
            "Expected Scryfall list JSON or object with data list"
        ));
    };

    let mut cards = Vec::new();
    for item in records {
        let name = item
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let type_line = first_face_text(&item, "type_line");
        let legalities = item
            .get("legalities")
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .map(|(key, value)| {
                        (
                            key.clone(),
                            value
                                .as_str()
                                .map(str::to_string)
                                .unwrap_or_else(|| value.to_string()),
                        )
                    })
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        cards.push(Card {
            name: name.clone(),
            mana_cost: first_face_text(&item, "mana_cost"),
            mana_value: item
                .get("cmc")
                .or_else(|| item.get("mana_value"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0),
            colors: as_vec(item.get("colors")),
            color_identity: as_vec(item.get("color_identity")),
            type_line: type_line.clone(),
            oracle_text: first_face_text(&item, "oracle_text"),
            keywords: as_vec(item.get("keywords")),
            rarity: item
                .get("rarity")
                .and_then(Value::as_str)
                .unwrap_or("common")
                .to_string(),
            set_code: item
                .get("set")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_uppercase(),
            collector_number: item
                .get("collector_number")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            legalities,
            games: as_vec(item.get("games")),
            arena_id: item.get("arena_id").and_then(Value::as_i64),
            is_basic_land: type_line.starts_with("Basic Land") || is_basic_land_name(&name),
            is_digital: item
                .get("digital")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            is_rebalanced: name.to_lowercase().contains("rebalanced"),
        });
    }
    Ok(cards)
}
