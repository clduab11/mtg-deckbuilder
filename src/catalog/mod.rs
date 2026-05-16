use crate::core::models::Card;
use crate::core::normalization::is_basic_land_name;
use crate::ingest::card_database::CardDatabase;
use anyhow::{Result, anyhow};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
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
    #[serde(alias = "Name", alias = "card_name", alias = "Card Name")]
    pub name: String,
    #[serde(default, alias = "Mana Cost", alias = "manaCost")]
    pub mana_cost: String,
    #[serde(default, alias = "CMC", alias = "cmc", alias = "manaValue")]
    pub mana_value: f64,
    #[serde(default, deserialize_with = "deserialize_symbol_list")]
    pub colors: Vec<String>,
    #[serde(
        default,
        alias = "colorIdentity",
        alias = "Color Identity",
        deserialize_with = "deserialize_symbol_list"
    )]
    pub color_identity: Vec<String>,
    #[serde(default, alias = "Type", alias = "type", alias = "typeLine")]
    pub type_line: String,
    #[serde(default, alias = "oracleText")]
    pub oracle_text: String,
    #[serde(default, deserialize_with = "deserialize_symbol_list")]
    pub keywords: Vec<String>,
    #[serde(default, alias = "Rarity")]
    pub rarity: String,
    #[serde(default, alias = "Set", alias = "set", alias = "setCode")]
    pub set_code: String,
    #[serde(default, alias = "collectorNumber", alias = "Number")]
    pub collector_number: String,
    #[serde(default)]
    pub legalities: BTreeMap<String, String>,
    #[serde(default, deserialize_with = "deserialize_symbol_list")]
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
        let type_line = if self.type_line.trim().is_empty() {
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
            rarity: if self.rarity.trim().is_empty() {
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

fn deserialize_symbol_list<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(match value {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Value::String(item) => vec![item],
        _ => Vec::new(),
    })
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

fn ensure_catalog_v1(schema_version: &str) -> Result<()> {
    if schema_version == "catalog.v1" {
        Ok(())
    } else {
        Err(anyhow!(
            "Unsupported catalog schema_version {schema_version:?}; expected \"catalog.v1\""
        ))
    }
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
        ensure_catalog_v1(&document.schema_version)?;
        return Ok(from_records(document.cards, warnings));
    }
    if value.get("data").and_then(Value::as_array).is_some()
        || value
            .as_array()
            .is_some_and(|items| looks_like_scryfall(items.first()))
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
        ensure_catalog_v1(&document.schema_version)?;
        return Ok(from_records(document.cards, warnings));
    }
    let records: Vec<CatalogCardRecord> = serde_json::from_value(value)?;
    Ok(from_records(records, warnings))
}

fn looks_like_scryfall(value: Option<&Value>) -> bool {
    value.is_some_and(|item| {
        item.get("object").is_some()
            || item.get("cmc").is_some()
            || item.get("collector_number").is_some()
            || item.get("card_faces").is_some()
    })
}

fn load_jsonl_cards(path: &Path, warnings: &mut Vec<String>) -> Result<Vec<Card>> {
    let mut records = Vec::new();
    let file = std::fs::File::open(path)?;
    for (idx, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: CatalogCardRecord = serde_json::from_str(&line)
            .map_err(|error| anyhow!("JSONL line {}: {error}", idx + 1))?;
        records.push(record);
    }
    Ok(from_records(records, warnings))
}

fn load_yaml_cards(path: &Path, warnings: &mut Vec<String>) -> Result<Vec<Card>> {
    let text = std::fs::read_to_string(path)?;
    let document: CatalogDocument = serde_yml::from_str(&text)?;
    ensure_catalog_v1(&document.schema_version)?;
    Ok(from_records(document.cards, warnings))
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y"
    )
}

fn parse_i64_option(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        trimmed.parse::<i64>().ok()
    }
}

fn parse_csv_list(value: &str) -> Vec<String> {
    value
        .split(['|', ';', ','])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_legalities(value: &str) -> BTreeMap<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return BTreeMap::new();
    }
    if let Ok(map) = serde_json::from_str::<BTreeMap<String, String>>(trimmed) {
        return map;
    }

    trimmed
        .split([';', ','])
        .filter_map(|part| {
            let (format, status) = part.split_once(':').or_else(|| part.split_once('='))?;
            let format = format.trim().to_ascii_lowercase();
            let status = status.trim().to_ascii_lowercase();
            (!format.is_empty() && !status.is_empty()).then_some((format, status))
        })
        .collect()
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
        let mut legalities = parse_legalities(&get(&["legalities", "Legalities"]));
        for format in [
            "standard",
            "alchemy",
            "historic",
            "explorer",
            "pioneer",
            "modern",
            "legacy",
            "vintage",
            "commander",
            "brawl",
        ] {
            let value = get(&[
                format,
                &format!("legal_{format}"),
                &format!("legality_{format}"),
            ]);
            if !value.is_empty() {
                legalities.insert(format.to_string(), value.to_ascii_lowercase());
            }
        }
        records.push(CatalogCardRecord {
            name,
            mana_cost: get(&["mana_cost", "Mana Cost", "manaCost"]),
            mana_value: get(&["mana_value", "manaValue", "cmc", "CMC"])
                .parse::<f64>()
                .unwrap_or(0.0),
            colors: normalize_symbols(vec![get(&["colors", "Colors"])]),
            color_identity: normalize_symbols(vec![get(&["color_identity", "Color Identity"])]),
            type_line: get(&["type_line", "Type", "type", "typeLine"]),
            oracle_text: get(&["oracle_text", "Oracle Text", "oracleText"]),
            keywords: parse_csv_list(&get(&["keywords", "Keywords"])),
            rarity: get(&["rarity", "Rarity"]),
            set_code: get(&["set_code", "set", "Set", "setCode"]),
            collector_number: get(&["collector_number", "collectorNumber", "Number"]),
            legalities,
            games: parse_csv_list(&get(&["games", "Games"]))
                .into_iter()
                .map(|game| game.to_ascii_lowercase())
                .collect(),
            arena_id: parse_i64_option(&get(&["arena_id", "Arena ID", "arenaId"])),
            is_digital: parse_bool(&get(&["is_digital", "digital", "Digital", "isDigital"])),
            is_rebalanced: parse_bool(&get(&[
                "is_rebalanced",
                "rebalanced",
                "Rebalanced",
                "isRebalanced",
            ])),
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
