use crate::core::models::Card;
use crate::core::normalization::is_basic_land_name;
use crate::ingest::card_database::CardDatabase;
use anyhow::{Result, anyhow};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct CatalogDocument {
    #[serde(default = "default_catalog_schema")]
    pub schema_version: String,
    #[serde(default)]
    pub cards: Vec<CatalogCardRecord>,
}

fn default_catalog_schema() -> String {
    "catalog.v1".to_string()
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
pub struct CatalogCardRecord {
    #[serde(alias = "Name", alias = "card_name")]
    pub name: String,
    #[serde(default, alias = "Mana Cost", alias = "manaCost")]
    pub mana_cost: String,
    #[serde(default, alias = "CMC", alias = "cmc", alias = "manaValue")]
    pub mana_value: f64,
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default, alias = "colorIdentity")]
    pub color_identity: Vec<String>,
    #[serde(default, alias = "Type", alias = "type")]
    pub type_line: String,
    #[serde(default, alias = "oracleText")]
    pub oracle_text: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default, alias = "Rarity")]
    pub rarity: String,
    #[serde(default, alias = "Set", alias = "set")]
    pub set_code: String,
    #[serde(default, alias = "collectorNumber")]
    pub collector_number: String,
    #[serde(default)]
    pub legalities: BTreeMap<String, String>,
    #[serde(default)]
    pub games: Vec<String>,
    #[serde(default)]
    pub arena_id: Option<i64>,
    #[serde(default)]
    pub is_basic_land: bool,
    #[serde(default, alias = "digital")]
    pub is_digital: bool,
    #[serde(default)]
    pub is_rebalanced: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct CatalogLoadReport {
    pub schema_version: String,
    pub source_path: String,
    pub format: String,
    pub card_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct LoadedCatalog {
    pub database: CardDatabase,
    pub report: CatalogLoadReport,
}

impl CatalogCardRecord {
    pub fn into_card(self) -> Card {
        let type_line = if self.type_line.is_empty() {
            "Unknown".to_string()
        } else {
            self.type_line
        };
        let name = self.name.trim().to_string();
        let is_basic =
            self.is_basic_land || type_line.contains("Basic Land") || is_basic_land_name(&name);
        Card {
            name,
            mana_cost: self.mana_cost,
            mana_value: self.mana_value,
            colors: normalize_symbols(self.colors),
            color_identity: normalize_symbols(self.color_identity),
            type_line,
            oracle_text: self.oracle_text,
            keywords: self.keywords,
            rarity: if self.rarity.is_empty() {
                "common".to_string()
            } else {
                self.rarity.to_lowercase()
            },
            set_code: self.set_code.to_uppercase(),
            collector_number: self.collector_number,
            legalities: self.legalities,
            games: if self.games.is_empty() {
                vec!["arena".to_string()]
            } else {
                self.games
            },
            arena_id: self.arena_id,
            is_basic_land: is_basic,
            is_digital: self.is_digital,
            is_rebalanced: self.is_rebalanced,
        }
    }
}

fn normalize_symbols(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .flat_map(|value| {
            value
                .split(['|', ';', ',', ' '])
                .filter(|part| !part.trim().is_empty())
                .map(|part| part.trim().to_uppercase())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn format_from_path(path: &Path, explicit: Option<&str>) -> Result<String> {
    if let Some(format) = explicit.filter(|format| *format != "auto") {
        return Ok(format.to_lowercase());
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "yml" => "yaml".to_string(),
            other => other.to_string(),
        })
        .ok_or_else(|| {
            anyhow!(
                "Could not infer catalog format from path: {}",
                path.display()
            )
        })
}

pub fn load_catalog_auto(path: impl AsRef<Path>) -> Result<LoadedCatalog> {
    load_catalog(path, None)
}

pub fn load_catalog(
    path: impl AsRef<Path>,
    explicit_format: Option<&str>,
) -> Result<LoadedCatalog> {
    let path = path.as_ref();
    let format = format_from_path(path, explicit_format)?;
    let mut warnings = Vec::new();
    let cards = match format.as_str() {
        "csv" => load_csv_cards(path, &mut warnings)?,
        "json" => load_json_cards(path, &mut warnings)?,
        "jsonl" | "ndjson" => load_jsonl_cards(path, &mut warnings)?,
        "yaml" => load_yaml_cards(path, &mut warnings)?,
        other => return Err(anyhow!("Unsupported catalog format: {other}")),
    };
    let card_count = cards.len();
    let database = CardDatabase::from_cards(cards, Some(path.to_string_lossy().to_string()));
    Ok(LoadedCatalog {
        database,
        report: CatalogLoadReport {
            schema_version: "catalog-load-report.v1".to_string(),
            source_path: path.to_string_lossy().to_string(),
            format,
            card_count,
            warnings,
        },
    })
}

fn from_records(records: Vec<CatalogCardRecord>, warnings: &mut Vec<String>) -> Vec<Card> {
    records
        .into_iter()
        .enumerate()
        .filter_map(|(idx, record)| {
            if record.name.trim().is_empty() {
                warnings.push(format!("row {} skipped: missing card name", idx + 1));
                None
            } else {
                Some(record.into_card())
            }
        })
        .collect()
}

fn load_json_cards(path: &Path, warnings: &mut Vec<String>) -> Result<Vec<Card>> {
    let text = std::fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text)?;
    if value.get("schema_version").is_some() && value.get("cards").is_some() {
        let document: CatalogDocument = serde_json::from_value(value)?;
        return Ok(from_records(document.cards, warnings));
    }
    if value.get("data").and_then(Value::as_array).is_some()
        || value.as_array().is_some_and(|items| {
            items
                .first()
                .and_then(|item| item.get("object"))
                .and_then(Value::as_str)
                .is_some()
        })
    {
        return crate::ingest::scryfall_loader::load_scryfall_cards(path);
    }
    if value
        .get("data")
        .and_then(Value::as_object)
        .is_some_and(|data| data.values().any(|set| set.get("cards").is_some()))
    {
        return crate::ingest::mtgjson_loader::load_mtgjson_cards(path);
    }
    if value.get("cards").is_some() {
        let document: CatalogDocument = serde_json::from_value(value)?;
        return Ok(from_records(document.cards, warnings));
    }
    let records: Vec<CatalogCardRecord> = serde_json::from_value(value)?;
    Ok(from_records(records, warnings))
}

fn load_jsonl_cards(path: &Path, warnings: &mut Vec<String>) -> Result<Vec<Card>> {
    let mut records = Vec::new();
    for (idx, line) in std::fs::read_to_string(path)?.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let record: CatalogCardRecord = serde_json::from_str(line)
            .map_err(|error| anyhow!("JSONL line {}: {error}", idx + 1))?;
        records.push(record);
    }
    Ok(from_records(records, warnings))
}

fn load_yaml_cards(path: &Path, warnings: &mut Vec<String>) -> Result<Vec<Card>> {
    let text = std::fs::read_to_string(path)?;
    let document: CatalogDocument = serde_yaml::from_str(&text)?;
    Ok(from_records(document.cards, warnings))
}

fn load_csv_cards(path: &Path, warnings: &mut Vec<String>) -> Result<Vec<Card>> {
    let mut reader = csv::Reader::from_path(path)?;
    let headers = reader.headers()?.clone();
    let mut records = Vec::new();
    for (idx, row) in reader.records().enumerate() {
        let row = row?;
        let get = |aliases: &[&str]| -> String {
            aliases
                .iter()
                .find_map(|alias| {
                    headers
                        .iter()
                        .position(|header| header.eq_ignore_ascii_case(alias))
                        .and_then(|pos| row.get(pos))
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                })
                .unwrap_or_default()
        };
        let name = get(&["name", "Name", "card_name", "Card Name"]);
        if name.is_empty() {
            warnings.push(format!("CSV row {} skipped: missing card name", idx + 1));
            continue;
        }
        records.push(CatalogCardRecord {
            name,
            mana_cost: get(&["mana_cost", "Mana Cost", "manaCost"]),
            mana_value: get(&["mana_value", "manaValue", "cmc", "CMC"])
                .parse::<f64>()
                .unwrap_or(0.0),
            colors: normalize_symbols(vec![get(&["colors", "Colors"])]),
            color_identity: normalize_symbols(vec![get(&["color_identity", "Color Identity"])]),
            type_line: get(&["type_line", "Type", "type"]),
            oracle_text: get(&["oracle_text", "Oracle Text"]),
            keywords: normalize_symbols(vec![get(&["keywords", "Keywords"])]),
            rarity: get(&["rarity", "Rarity"]),
            set_code: get(&["set_code", "set", "Set"]),
            collector_number: get(&["collector_number", "collectorNumber", "Number"]),
            games: normalize_symbols(vec![get(&["games", "Games"])]),
            ..CatalogCardRecord::default()
        });
    }
    Ok(from_records(records, warnings))
}

pub fn schema_json(name: &str) -> Result<Value> {
    let schema = match name {
        "catalog" => serde_json::to_value(schema_for!(CatalogDocument))?,
        "catalog-load-report" => serde_json::to_value(schema_for!(CatalogLoadReport))?,
        other => return Err(anyhow!("Unsupported catalog schema: {other}")),
    };
    Ok(schema)
}

pub fn example_path(format: &str) -> PathBuf {
    PathBuf::from(format!("examples/sample_catalog.{format}"))
}
