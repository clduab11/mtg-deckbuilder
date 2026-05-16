use anyhow::{Result, anyhow};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct AnalysisReport {
    pub schema_version: String,
    pub generated_by: String,
    pub assumptions: Vec<String>,
    pub validation: Value,
    pub opening_hand: Value,
    pub early_turns: Value,
    pub features: Value,
    pub consistency: Value,
    pub source_hashes: BTreeMap<String, String>,
}

pub fn render_json(report: &AnalysisReport) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

pub fn render_markdown(report: &AnalysisReport) -> String {
    let status = report
        .validation
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("UNKNOWN");
    let consistency = report
        .consistency
        .get("consistency_score")
        .and_then(Value::as_f64)
        .map(|score| score.to_string())
        .unwrap_or_else(|| "n/a".to_string());
    let mut lines = vec![
        "# MTG Deck Analysis Report".to_string(),
        String::new(),
        format!("- Schema: {}", report.schema_version),
        format!("- Validation: {status}"),
        format!("- Consistency score: {consistency}"),
        String::new(),
        "## Assumptions".to_string(),
    ];
    for assumption in &report.assumptions {
        lines.push(format!("- {assumption}"));
    }
    lines.extend([String::new(), "## Source Hashes".to_string()]);
    for (name, hash) in &report.source_hashes {
        lines.push(format!("- {name}: `{hash}`"));
    }
    lines.push(String::new());
    lines.join("\n")
}

pub fn render_csv(report: &AnalysisReport) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(["metric", "value"])?;
    writer.write_record([
        "validation_status",
        report
            .validation
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN"),
    ])?;
    if let Some(score) = report
        .consistency
        .get("consistency_score")
        .and_then(Value::as_f64)
    {
        writer.write_record(["consistency_score", &score.to_string()])?;
    }
    for (name, hash) in &report.source_hashes {
        writer.write_record([format!("source_hash.{name}"), hash.clone()])?;
    }
    Ok(String::from_utf8(writer.into_inner()?)?)
}

pub fn render_report(report: &AnalysisReport, format: &str) -> Result<String> {
    match format {
        "json" => render_json(report),
        "markdown" | "md" => Ok(render_markdown(report)),
        "csv" => render_csv(report),
        other => Err(anyhow!("Unsupported report format: {other}")),
    }
}
