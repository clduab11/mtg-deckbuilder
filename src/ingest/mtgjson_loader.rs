use crate::core::models::Card;
use crate::core::normalization::is_basic_land_name;
use anyhow::Result;
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

pub fn load_mtgjson_cards(path: impl AsRef<Path>) -> Result<Vec<Card>> {
    let data: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let payload = data.get("data").unwrap_or(&data);
    let mut records: Vec<(String, Value)> = Vec::new();

    if let Some(cards) = payload.get("cards").and_then(Value::as_array) {
        let set_code = payload
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        for card in cards {
            records.push((set_code.clone(), card.clone()));
        }
    } else if let Some(sets) = payload.as_object() {
        for (set_code, set_obj) in sets {
            if let Some(cards) = set_obj.get("cards").and_then(Value::as_array) {
                for card in cards {
                    records.push((set_code.clone(), card.clone()));
                }
            }
        }
    }

    let mut cards = Vec::new();
    for (set_code, item) in records {
        let name = item
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let legalities = item
            .get("legalities")
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .map(|(key, value)| {
                        (
                            key.to_lowercase(),
                            value.as_str().unwrap_or("").to_lowercase(),
                        )
                    })
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        let type_line = item
            .get("type")
            .or_else(|| item.get("typeLine"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        cards.push(Card {
            name: name.clone(),
            mana_cost: item
                .get("manaCost")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            mana_value: item
                .get("manaValue")
                .or_else(|| item.get("convertedManaCost"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0),
            colors: as_vec(item.get("colors")),
            color_identity: as_vec(item.get("colorIdentity")),
            type_line: type_line.clone(),
            oracle_text: item
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            keywords: as_vec(item.get("keywords")),
            rarity: item
                .get("rarity")
                .and_then(Value::as_str)
                .unwrap_or("common")
                .to_lowercase(),
            set_code: item
                .get("setCode")
                .and_then(Value::as_str)
                .unwrap_or(&set_code)
                .to_uppercase(),
            collector_number: item
                .get("number")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            legalities,
            games: if item
                .get("isArena")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                vec!["arena".to_string()]
            } else {
                Vec::new()
            },
            arena_id: None,
            is_basic_land: type_line.contains("Basic Land") || is_basic_land_name(&name),
            is_digital: item
                .get("isOnlineOnly")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            is_rebalanced: item
                .get("isRebalanced")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        });
    }
    Ok(cards)
}
