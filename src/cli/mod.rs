use crate::analysis::{AnalysisInputs, build_analysis_report as build_shared_analysis_report};
use crate::catalog::{load_catalog, load_catalog_auto};
use crate::eval::consistency::score_consistency;
use crate::export::arena::format_arena_decklist;
use crate::features::deck_features::extract_deck_features;
use crate::ingest::card_database::CardDatabase;
use crate::ingest::collection_parser::parse_collection_csv;
use crate::ingest::decklist_importer::parse_arena_decklist_file;
use crate::llm::LlmAnalysisArtifact;
use crate::observability::deck_hash::deck_hash;
use crate::observability::experiment_logger::write_experiment;
use crate::observability::source_snapshot::file_sha256;
use crate::report::{AnalysisReport, render_report};
use crate::result_log::load_result_log;
use crate::rules::validator::DeckValidator;
use crate::sim::bo1::Bo1Config;
use crate::sim::bo3::Bo3Config;
use crate::sim::early_turns::simulate_first_three_turns;
use crate::sim::opening_hand::simulate_opening_hands;
use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "mtgdeckbuilder")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Validate {
        #[arg(long)]
        deck: String,
        #[arg(long)]
        cards: String,
        #[arg(long)]
        collection: Option<String>,
        #[arg(long)]
        format: String,
        #[arg(long)]
        craft_mode: bool,
    },
    ImportCatalog {
        #[arg(long)]
        input: String,
        #[arg(long, default_value = "auto")]
        format: String,
    },
    ImportResultLog {
        #[arg(long)]
        input: String,
        #[arg(long, default_value = "auto")]
        format: String,
    },
    Export {
        #[arg(long)]
        deck: String,
    },
    SimulateOpening {
        #[arg(long)]
        deck: String,
        #[arg(long)]
        cards: String,
        #[arg(long, default_value = "arena_n2")]
        mode: String,
        #[arg(long, default_value_t = 1000)]
        trials: u32,
        #[arg(long, default_value_t = 1)]
        seed: u64,
    },
    Simulate {
        #[arg(long)]
        deck: String,
        #[arg(long)]
        cards: String,
        #[arg(long, default_value = "bo1")]
        queue: String,
        #[arg(long, default_value_t = 1000)]
        trials: u32,
        #[arg(long, default_value_t = 1)]
        seed: u64,
    },
    Report {
        #[arg(long)]
        deck: String,
        #[arg(long)]
        cards: String,
        #[arg(long)]
        collection: Option<String>,
        #[arg(long)]
        format: String,
        #[arg(long, default_value = "json")]
        output: String,
        #[arg(long)]
        result_log: Option<String>,
        #[arg(long, default_value = "auto")]
        result_log_format: String,
        #[arg(long, default_value_t = 1000)]
        trials: u32,
        #[arg(long, default_value_t = 1)]
        seed: u64,
    },
    Schema {
        #[arg(long)]
        name: String,
    },
    LlmArtifact {
        #[arg(long)]
        deck: String,
        #[arg(long)]
        cards: String,
        #[arg(long)]
        format: String,
        #[arg(long, default_value_t = 1000)]
        trials: u32,
        #[arg(long, default_value_t = 1)]
        seed: u64,
    },
    EvalSmoke {
        #[arg(long)]
        deck: String,
        #[arg(long)]
        cards: String,
        #[arg(long)]
        collection: Option<String>,
        #[arg(long)]
        format: String,
        #[arg(long)]
        craft_mode: bool,
        #[arg(long, default_value_t = 1000)]
        trials: u32,
        #[arg(long, default_value_t = 1)]
        seed: u64,
    },
}

pub fn run_exit_code() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("ERROR: {error}");
            ExitCode::from(2)
        }
    }
}

pub fn run() -> Result<u8> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Validate {
            deck,
            cards,
            collection,
            format,
            craft_mode,
        } => cmd_validate(&deck, &cards, collection.as_deref(), &format, craft_mode),
        Commands::ImportCatalog { input, format } => cmd_import_catalog(&input, &format),
        Commands::ImportResultLog { input, format } => cmd_import_result_log(&input, &format),
        Commands::Export { deck } => cmd_export(&deck),
        Commands::SimulateOpening {
            deck,
            cards,
            mode,
            trials,
            seed,
        } => cmd_simulate_opening(&deck, &cards, &mode, trials, seed),
        Commands::Simulate {
            deck,
            cards,
            queue,
            trials,
            seed,
        } => cmd_simulate(&deck, &cards, &queue, trials, seed),
        Commands::Report {
            deck,
            cards,
            collection,
            format,
            output,
            result_log,
            result_log_format,
            trials,
            seed,
        } => cmd_report(
            AnalysisReportInputs {
                deck_path: &deck,
                cards_path: &cards,
                collection_path: collection.as_deref(),
                format_name: &format,
                result_log_path: result_log.as_deref(),
                result_log_format: Some(&result_log_format),
                trials,
                seed,
            },
            &output,
        ),
        Commands::Schema { name } => cmd_schema(&name),
        Commands::LlmArtifact {
            deck,
            cards,
            format,
            trials,
            seed,
        } => cmd_llm_artifact(&deck, &cards, &format, trials, seed),
        Commands::EvalSmoke {
            deck,
            cards,
            collection,
            format,
            craft_mode,
            trials,
            seed,
        } => cmd_eval_smoke(
            &deck,
            &cards,
            collection.as_deref(),
            &format,
            craft_mode,
            trials,
            seed,
        ),
    }
}

fn load_card_db(path: &str) -> Result<CardDatabase> {
    Ok(load_catalog_auto(path)?.database)
}

fn print_json(value: impl Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn cmd_validate(
    deck_path: &str,
    cards_path: &str,
    collection_path: Option<&str>,
    format_name: &str,
    craft_mode: bool,
) -> Result<u8> {
    let deck = parse_arena_decklist_file(deck_path)?;
    let db = load_card_db(cards_path)?;
    let collection = collection_path
        .map(|path| parse_collection_csv(path, None, None))
        .transpose()?;
    let report =
        DeckValidator::new(db).validate(&deck, format_name, collection.as_ref(), craft_mode, None);
    let code = if report.ok() { 0 } else { 1 };
    print_json(report)?;
    Ok(code)
}

fn cmd_import_catalog(path: &str, format: &str) -> Result<u8> {
    let loaded = load_catalog(path, Some(format))?;
    print_json(loaded.report)?;
    Ok(0)
}

fn cmd_import_result_log(path: &str, format: &str) -> Result<u8> {
    let loaded = load_result_log(path, Some(format))?;
    print_json(loaded.report)?;
    Ok(0)
}

fn cmd_export(deck_path: &str) -> Result<u8> {
    let deck = parse_arena_decklist_file(deck_path)?;
    print!("{}", format_arena_decklist(&deck));
    Ok(0)
}

fn cmd_simulate_opening(
    deck_path: &str,
    cards_path: &str,
    mode: &str,
    trials: u32,
    seed: u64,
) -> Result<u8> {
    let deck = parse_arena_decklist_file(deck_path)?;
    let db = load_card_db(cards_path)?;
    let result = simulate_opening_hands(&deck, &db, mode, trials, seed, 2)?;
    print_json(result)?;
    Ok(0)
}

fn cmd_simulate(
    deck_path: &str,
    cards_path: &str,
    queue: &str,
    trials: u32,
    seed: u64,
) -> Result<u8> {
    let deck = parse_arena_decklist_file(deck_path)?;
    let db = load_card_db(cards_path)?;
    let (mode, max_mulligans, config) = match queue {
        "bo1" => {
            let config = Bo1Config::default();
            (
                config.opening_hand_mode.clone(),
                config.max_mulligans,
                serde_json::to_value(config)?,
            )
        }
        "bo3" => {
            let config = Bo3Config::default();
            (
                config.opening_hand_mode.clone(),
                config.max_mulligans,
                serde_json::to_value(config)?,
            )
        }
        other => return Err(anyhow!("Unsupported queue: {other}; use bo1 or bo3")),
    };
    let opening = simulate_opening_hands(&deck, &db, &mode, trials, seed, max_mulligans)?;
    let early = simulate_first_three_turns(&deck, &db, trials, seed)?;
    print_json(json!({
        "schema_version": "simulation-result.v1",
        "queue": queue,
        "config": config,
        "opening_hand": opening,
        "early_turns": early,
        "seed": seed,
        "trials": trials
    }))?;
    Ok(0)
}

struct AnalysisReportInputs<'a> {
    deck_path: &'a str,
    cards_path: &'a str,
    collection_path: Option<&'a str>,
    format_name: &'a str,
    result_log_path: Option<&'a str>,
    result_log_format: Option<&'a str>,
    trials: u32,
    seed: u64,
}

fn build_analysis_report(inputs: &AnalysisReportInputs<'_>) -> Result<AnalysisReport> {
    let deck = parse_arena_decklist_file(inputs.deck_path)?;
    let catalog = load_catalog_auto(inputs.cards_path)?;
    let collection = inputs
        .collection_path
        .map(|path| parse_collection_csv(path, None, None))
        .transpose()?;
    let mut source_hashes = BTreeMap::new();
    source_hashes.insert("cards".to_string(), file_sha256(inputs.cards_path)?);
    if let Some(collection_path) = inputs.collection_path {
        source_hashes.insert("collection".to_string(), file_sha256(collection_path)?);
    }
    source_hashes.insert("deck".to_string(), file_sha256(inputs.deck_path)?);
    let result_log = inputs
        .result_log_path
        .map(|path| {
            let loaded = load_result_log(path, inputs.result_log_format)?;
            source_hashes.insert("result_log".to_string(), file_sha256(path)?);
            Ok::<_, anyhow::Error>(loaded)
        })
        .transpose()?;

    build_shared_analysis_report(AnalysisInputs {
        deck,
        catalog,
        collection,
        result_log,
        format_name: inputs.format_name.to_string(),
        queue: "bo1".to_string(),
        trials: inputs.trials,
        seed: inputs.seed,
        source_hashes,
    })
}

fn cmd_report(inputs: AnalysisReportInputs<'_>, output: &str) -> Result<u8> {
    let report = build_analysis_report(&inputs)?;
    print!("{}", render_report(&report, output)?);
    Ok(0)
}

fn cmd_schema(name: &str) -> Result<u8> {
    let schema = match name {
        "catalog" | "catalog-load-report" => crate::catalog::schema_json(name)?,
        "result-log" | "result-log-load-report" => crate::result_log::schema_json(name)?,
        "llm" | "llm-report" => crate::llm::schema_json()?,
        "api-deck-validate-request" => crate::api_contract::schema_json("deck-validate-request")?,
        "api-simulation-run-request" => crate::api_contract::schema_json("simulation-run-request")?,
        "api-export-request" => crate::api_contract::schema_json("export-request")?,
        other => return Err(anyhow!("Unsupported schema name: {other}")),
    };
    print_json(schema)?;
    Ok(0)
}

fn cmd_llm_artifact(
    deck_path: &str,
    cards_path: &str,
    format_name: &str,
    trials: u32,
    seed: u64,
) -> Result<u8> {
    let report = build_analysis_report(&AnalysisReportInputs {
        deck_path,
        cards_path,
        collection_path: None,
        format_name,
        result_log_path: None,
        result_log_format: None,
        trials,
        seed,
    })?;
    let artifact = LlmAnalysisArtifact::new(serde_json::to_value(report)?);
    print_json(artifact)?;
    Ok(0)
}

fn cmd_eval_smoke(
    deck_path: &str,
    cards_path: &str,
    collection_path: Option<&str>,
    format_name: &str,
    craft_mode: bool,
    trials: u32,
    seed: u64,
) -> Result<u8> {
    let deck = parse_arena_decklist_file(deck_path)?;
    let db = load_card_db(cards_path)?;
    let collection = collection_path
        .map(|path| parse_collection_csv(path, None, None))
        .transpose()?;
    let validation = DeckValidator::new(db.clone()).validate(
        &deck,
        format_name,
        collection.as_ref(),
        craft_mode,
        None,
    );
    let opening = simulate_opening_hands(&deck, &db, "arena_n2", trials, seed, 2)?;
    let early = simulate_first_three_turns(&deck, &db, trials, seed)?;
    let features = extract_deck_features(&deck, &db);
    let consistency = score_consistency(&opening, &early);

    let mut source_hashes = Map::new();
    source_hashes.insert("cards".to_string(), json!(file_sha256(cards_path)?));
    if let Some(collection_path) = collection_path {
        source_hashes.insert(
            "collection".to_string(),
            json!(file_sha256(collection_path)?),
        );
    }
    source_hashes.insert("deck".to_string(), json!(file_sha256(deck_path)?));

    let payload = json!({
        "assumptions": [
            "Fixture card data is for smoke testing only unless replaced with current authoritative data.",
            "No metagame claims are made by eval-smoke."
        ],
        "consistency": consistency,
        "deck_hash": deck_hash(&deck),
        "early_turns": early,
        "features": features,
        "opening_hand": opening,
        "source_hashes": Value::Object(source_hashes),
        "validation": validation,
    });
    let record = write_experiment("experiments/eval_smoke.jsonl", payload.clone(), None)?;

    let mut output: BTreeMap<String, Value> = serde_json::from_value(payload)?;
    output.insert("experiment_id".to_string(), json!(record.experiment_id));
    print_json(&output)?;
    let validation_status = output
        .get("validation")
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("FAIL");
    Ok(if validation_status == "PASS" { 0 } else { 1 })
}
