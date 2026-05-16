use crate::catalog::LoadedCatalog;
use crate::core::models::{CollectionIndex, Decklist};
use crate::eval::consistency::score_consistency;
use crate::features::deck_features::extract_deck_features;
use crate::report::AnalysisReport;
use crate::result_log::{LoadedResultLog, summarize_loaded_result_log};
use crate::rules::validator::DeckValidator;
use crate::sim::bo1::Bo1Config;
use crate::sim::bo3::Bo3Config;
use crate::sim::early_turns::simulate_first_three_turns;
use crate::sim::opening_hand::simulate_opening_hands;
use anyhow::{Result, anyhow};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct AnalysisInputs {
    pub deck: Decklist,
    pub catalog: LoadedCatalog,
    pub collection: Option<CollectionIndex>,
    pub result_log: Option<LoadedResultLog>,
    pub format_name: String,
    pub queue: String,
    pub trials: u32,
    pub seed: u64,
    pub source_hashes: BTreeMap<String, String>,
}

struct QueueAnalysisConfig {
    opening_hand_mode: String,
    max_mulligans: u32,
    assumptions: Vec<String>,
}

fn queue_config(queue: &str) -> Result<QueueAnalysisConfig> {
    match queue {
        "bo1" => {
            let config = Bo1Config::default();
            Ok(QueueAnalysisConfig {
                opening_hand_mode: config.opening_hand_mode,
                max_mulligans: config.max_mulligans,
                assumptions: config.assumptions,
            })
        }
        "bo3" => {
            let config = Bo3Config::default();
            Ok(QueueAnalysisConfig {
                opening_hand_mode: config.opening_hand_mode,
                max_mulligans: config.max_mulligans,
                assumptions: config.assumptions,
            })
        }
        other => Err(anyhow!("Unsupported queue: {other}; use bo1 or bo3")),
    }
}

pub fn build_analysis_report(inputs: AnalysisInputs) -> Result<AnalysisReport> {
    let queue = queue_config(&inputs.queue)?;
    let db = inputs.catalog.database;
    let validation = DeckValidator::new(db.clone()).validate(
        &inputs.deck,
        &inputs.format_name,
        inputs.collection.as_ref(),
        false,
        None,
    );
    let opening = simulate_opening_hands(
        &inputs.deck,
        &db,
        &queue.opening_hand_mode,
        inputs.trials,
        inputs.seed,
        queue.max_mulligans,
    )?;
    let early = simulate_first_three_turns(&inputs.deck, &db, inputs.trials, inputs.seed)?;
    let features = extract_deck_features(&inputs.deck, &db);
    let consistency = score_consistency(&opening, &early);
    let result_log = inputs
        .result_log
        .as_ref()
        .map(summarize_loaded_result_log)
        .transpose()?;

    let mut assumptions = vec![
        "Bo1/Bo3-oriented offline analysis; no exact MTG Arena parity claim.".to_string(),
        "Fixture data is not authoritative legality or metagame data.".to_string(),
        "Uploaded or supplied content is processed as user-owned local data.".to_string(),
    ];
    assumptions.extend(queue.assumptions);
    assumptions.extend(inputs.catalog.report.warnings.iter().map(|warning| {
        format!(
            "Catalog warning from {}: {warning}",
            inputs.catalog.report.source_path
        )
    }));
    if let Some(collection) = &inputs.collection {
        assumptions.extend(
            collection
                .warnings
                .iter()
                .map(|warning| format!("Collection warning: {warning}")),
        );
    }
    if let Some(result_log) = &inputs.result_log {
        assumptions.extend(result_log.report.warnings.iter().map(|warning| {
            format!(
                "Result-log warning from {}: {warning}",
                result_log.report.source_path
            )
        }));
    }

    Ok(AnalysisReport {
        schema_version: "analysis-report.v1".to_string(),
        generated_by: "mtgdeckbuilder-rust".to_string(),
        assumptions,
        validation: serde_json::to_value(validation)?,
        opening_hand: serde_json::to_value(opening)?,
        early_turns: serde_json::to_value(early)?,
        features,
        consistency,
        result_log,
        source_hashes: inputs.source_hashes,
    })
}
