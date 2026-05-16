use crate::stats::{DraftPickRecord, GameRecord, summarize_constructed, summarize_draft_card};
use anyhow::{Result, anyhow};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, PartialEq)]
pub struct ResultLogDocument {
    pub schema_version: String,
    #[serde(default)]
    pub games: Vec<GameRecord>,
    #[serde(default)]
    pub draft_picks: Vec<DraftPickRecord>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, PartialEq, Eq)]
pub struct ResultLogLoadReport {
    pub schema_version: String,
    pub source_path: String,
    pub format: String,
    pub game_count: usize,
    pub draft_pick_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedResultLog {
    pub games: Vec<GameRecord>,
    pub draft_picks: Vec<DraftPickRecord>,
    pub report: ResultLogLoadReport,
}

fn ensure_result_log_v1(schema_version: &str) -> Result<()> {
    if schema_version == "result-log.v1" {
        Ok(())
    } else {
        Err(anyhow!(
            "Unsupported result-log schema_version {schema_version:?}; expected \"result-log.v1\""
        ))
    }
}

fn format_from_path(path: &Path, explicit: Option<&str>) -> Result<String> {
    if let Some(format) = explicit.filter(|format| *format != "auto") {
        return Ok(format.to_ascii_lowercase());
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .ok_or_else(|| {
            anyhow!(
                "Could not infer result-log format from path: {}",
                path.display()
            )
        })
}

pub fn load_result_log_auto(path: impl AsRef<Path>) -> Result<LoadedResultLog> {
    load_result_log(path, None)
}

pub fn load_result_log(
    path: impl AsRef<Path>,
    explicit_format: Option<&str>,
) -> Result<LoadedResultLog> {
    let path = path.as_ref();
    let format = format_from_path(path, explicit_format)?;
    let mut warnings = Vec::new();
    let (games, draft_picks) = match format.as_str() {
        "csv" => load_csv_result_log(path, &mut warnings)?,
        "json" => load_json_result_log(path)?,
        "jsonl" | "ndjson" => load_jsonl_result_log(path, &mut warnings)?,
        other => return Err(anyhow!("Unsupported result-log format: {other}")),
    };
    Ok(LoadedResultLog {
        report: ResultLogLoadReport {
            schema_version: "result-log-load-report.v1".to_string(),
            source_path: path.to_string_lossy().to_string(),
            format,
            game_count: games.len(),
            draft_pick_count: draft_picks.len(),
            warnings,
        },
        games,
        draft_picks,
    })
}

fn load_json_result_log(path: &Path) -> Result<(Vec<GameRecord>, Vec<DraftPickRecord>)> {
    let text = std::fs::read_to_string(path)?;
    let document: ResultLogDocument = serde_json::from_str(&text)?;
    ensure_result_log_v1(&document.schema_version)?;
    Ok((document.games, document.draft_picks))
}

fn load_jsonl_result_log(
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<(Vec<GameRecord>, Vec<DraftPickRecord>)> {
    let mut games = Vec::new();
    let mut draft_picks = Vec::new();
    let file = std::fs::File::open(path)?;
    for (idx, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            warnings.push(format!("JSONL line {} skipped: blank line", idx + 1));
            continue;
        }
        let value: Value = serde_json::from_str(&line)
            .map_err(|error| anyhow!("JSONL line {}: {error}", idx + 1))?;
        push_row(
            value_to_map(value, &format!("JSONL line {}", idx + 1))?,
            &format!("JSONL line {}", idx + 1),
            &mut games,
            &mut draft_picks,
        )?;
    }
    Ok((games, draft_picks))
}

fn load_csv_result_log(
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<(Vec<GameRecord>, Vec<DraftPickRecord>)> {
    let mut reader = csv::Reader::from_path(path)?;
    let headers = reader.headers()?.clone();
    let mut games = Vec::new();
    let mut draft_picks = Vec::new();
    for (idx, row) in reader.records().enumerate() {
        let row = row?;
        if row.iter().all(|value| value.trim().is_empty()) {
            warnings.push(format!("CSV row {} skipped: blank row", idx + 1));
            continue;
        }
        let mut map = BTreeMap::new();
        for (header, value) in headers.iter().zip(row.iter()) {
            map.insert(
                normalize_key(header),
                Value::String(value.trim().to_string()),
            );
        }
        push_row(
            map,
            &format!("CSV row {}", idx + 1),
            &mut games,
            &mut draft_picks,
        )?;
    }
    Ok((games, draft_picks))
}

fn push_row(
    map: BTreeMap<String, Value>,
    context: &str,
    games: &mut Vec<GameRecord>,
    draft_picks: &mut Vec<DraftPickRecord>,
) -> Result<()> {
    match normalize_record_type(&required_string(
        &map,
        &["record_type", "recordType"],
        context,
    )?)?
    .as_str()
    {
        "game" => games.push(parse_game_record(&map, context)?),
        "draft_pick" => draft_picks.push(parse_draft_pick_record(&map, context)?),
        record_type => {
            return Err(anyhow!(
                "{context}: unsupported record_type {record_type:?}; use \"game\" or \"draft_pick\""
            ));
        }
    }
    Ok(())
}

fn value_to_map(value: Value, context: &str) -> Result<BTreeMap<String, Value>> {
    let object = value
        .as_object()
        .ok_or_else(|| anyhow!("{context}: expected an object row"))?;
    Ok(object
        .iter()
        .map(|(key, value)| (normalize_key(key), value.clone()))
        .collect())
}

fn parse_game_record(map: &BTreeMap<String, Value>, context: &str) -> Result<GameRecord> {
    Ok(GameRecord {
        match_id: required_string(map, &["match_id", "matchId", "match"], context)?,
        game_number: required_u8(
            map,
            &["game_number", "gameNumber", "game", "game_num"],
            context,
        )?,
        queue: required_string(map, &["queue", "event_queue"], context)?.to_ascii_lowercase(),
        format_name: required_string(map, &["format_name", "formatName", "format"], context)?
            .to_ascii_lowercase(),
        play_draw: optional_string(map, &["play_draw", "playDraw", "play/draw"])
            .map(|value| normalize_play_draw(&value))
            .transpose()?,
        opponent_archetype: optional_string(
            map,
            &[
                "opponent_archetype",
                "opponentArchetype",
                "opponent",
                "matchup",
            ],
        ),
        won: required_bool(map, &["won", "win", "result", "outcome"], context)?,
        mulligans: required_u8(
            map,
            &["mulligans", "mulligan_count", "mulliganCount"],
            context,
        )?,
        sideboarded: required_bool(
            map,
            &["sideboarded", "sideboard", "post_board", "postBoard"],
            context,
        )?,
    })
}

fn parse_draft_pick_record(
    map: &BTreeMap<String, Value>,
    context: &str,
) -> Result<DraftPickRecord> {
    Ok(DraftPickRecord {
        draft_id: required_string(map, &["draft_id", "draftId", "draft"], context)?,
        card_name: required_string(map, &["card_name", "cardName", "card"], context)?,
        pack_number: required_u8(map, &["pack_number", "packNumber", "pack"], context)?,
        pick_number: required_u8(map, &["pick_number", "pickNumber", "pick"], context)?,
        seen_at_pick: required_u8(
            map,
            &["seen_at_pick", "seenAtPick", "last_seen_at", "lastSeenAt"],
            context,
        )?,
        taken: required_bool(map, &["taken", "picked"], context)?,
        opening_hand_games: required_u32(
            map,
            &["opening_hand_games", "openingHandGames", "oh_games"],
            context,
        )?,
        opening_hand_wins: required_u32(
            map,
            &["opening_hand_wins", "openingHandWins", "oh_wins"],
            context,
        )?,
        drawn_games: required_u32(map, &["drawn_games", "drawnGames"], context)?,
        drawn_wins: required_u32(map, &["drawn_wins", "drawnWins"], context)?,
        games: required_u32(map, &["games", "game_count", "gameCount"], context)?,
        wins: required_u32(map, &["wins", "win_count", "winCount"], context)?,
        trophies: required_u32(map, &["trophies", "trophy_count", "trophyCount"], context)?,
        events: required_u32(map, &["events", "event_count", "eventCount"], context)?,
        color_pair: optional_string(map, &["color_pair", "colorPair", "colors"]),
        archetype: optional_string(map, &["archetype", "draft_archetype", "draftArchetype"]),
        wheeled: required_bool(map, &["wheeled", "wheel"], context)?,
    })
}

fn normalize_key(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) => Some(value.trim().to_string()).filter(|value| !value.is_empty()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn optional_string(map: &BTreeMap<String, Value>, aliases: &[&str]) -> Option<String> {
    aliases.iter().find_map(|alias| {
        let key = normalize_key(alias);
        map.get(key.as_str()).and_then(value_as_string)
    })
}

fn required_string(
    map: &BTreeMap<String, Value>,
    aliases: &[&str],
    context: &str,
) -> Result<String> {
    optional_string(map, aliases)
        .ok_or_else(|| anyhow!("{context}: missing required field {}", aliases[0]))
}

fn required_u8(map: &BTreeMap<String, Value>, aliases: &[&str], context: &str) -> Result<u8> {
    let value = required_string(map, aliases, context)?;
    value
        .parse::<u8>()
        .map_err(|error| anyhow!("{context}: invalid {} value {value:?}: {error}", aliases[0]))
}

fn required_u32(map: &BTreeMap<String, Value>, aliases: &[&str], context: &str) -> Result<u32> {
    let value = required_string(map, aliases, context)?;
    value
        .parse::<u32>()
        .map_err(|error| anyhow!("{context}: invalid {} value {value:?}: {error}", aliases[0]))
}

fn required_bool(map: &BTreeMap<String, Value>, aliases: &[&str], context: &str) -> Result<bool> {
    let value = required_string(map, aliases, context)?;
    parse_bool(&value).ok_or_else(|| anyhow!("{context}: invalid {} boolean {value:?}", aliases[0]))
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "win" | "won" | "picked" | "taken" => Some(true),
        "0" | "false" | "no" | "n" | "loss" | "lost" | "lose" | "not_taken" => Some(false),
        _ => None,
    }
}

fn normalize_record_type(value: &str) -> Result<String> {
    match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
        "game" => Ok("game".to_string()),
        "draft_pick" | "draftpick" => Ok("draft_pick".to_string()),
        other => Err(anyhow!(
            "unsupported record_type {other:?}; use \"game\" or \"draft_pick\""
        )),
    }
}

fn normalize_play_draw(value: &str) -> Result<String> {
    match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
        "play" | "on_play" | "otp" => Ok("play".to_string()),
        "draw" | "on_draw" | "otd" => Ok("draw".to_string()),
        other => Err(anyhow!(
            "unsupported play_draw value {other:?}; use \"play\" or \"draw\""
        )),
    }
}

pub fn summarize_loaded_result_log(loaded: &LoadedResultLog) -> Result<Value> {
    let constructed = (!loaded.games.is_empty())
        .then(|| serde_json::to_value(summarize_constructed(&loaded.games)))
        .transpose()?;

    let mut draft_cards = BTreeMap::new();
    for card_name in loaded
        .draft_picks
        .iter()
        .map(|record| record.card_name.clone())
        .collect::<BTreeSet<_>>()
    {
        draft_cards.insert(
            card_name.clone(),
            serde_json::to_value(summarize_draft_card(&card_name, &loaded.draft_picks))?,
        );
    }

    Ok(json!({
        "schema_version": "result-log-summary.v1",
        "source": loaded.report,
        "constructed": constructed,
        "draft_cards": draft_cards,
        "assumptions": [
            "Result-log records are user-supplied local data.",
            "Result-log summaries do not alter validation, seeded simulation, or deterministic outputs."
        ]
    }))
}

pub fn schema_json(name: &str) -> Result<Value> {
    let schema = match name {
        "result-log" => serde_json::to_value(schema_for!(ResultLogDocument))?,
        "result-log-load-report" => serde_json::to_value(schema_for!(ResultLogLoadReport))?,
        other => return Err(anyhow!("Unsupported result-log schema: {other}")),
    };
    Ok(schema)
}
