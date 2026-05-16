use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const ROUTES: &[(&str, &str)] = &[
    ("POST", "/deck/validate"),
    ("POST", "/simulation/run"),
    ("GET", "/simulation/{id}/status"),
    ("GET", "/simulation/{id}/results"),
    ("GET", "/simulation/{id}/report"),
    ("POST", "/export"),
];

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct DeckValidateRequest {
    pub deck_text: String,
    pub catalog_path: String,
    pub format_name: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct DeckValidateResponse {
    pub validation: Value,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct SimulationRunRequest {
    pub deck_text: String,
    pub catalog_path: String,
    pub queue: String,
    pub trials: u32,
    pub seed: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct SimulationRunResponse {
    pub simulation_id: String,
    pub status: String,
    pub assumptions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct SimulationStatusResponse {
    pub simulation_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct ExportRequest {
    pub deck_text: String,
    pub format: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct ExportResponse {
    pub body: String,
    pub content_type: String,
}

pub fn schema_json(name: &str) -> anyhow::Result<Value> {
    let schema = match name {
        "deck-validate-request" => serde_json::to_value(schema_for!(DeckValidateRequest))?,
        "simulation-run-request" => serde_json::to_value(schema_for!(SimulationRunRequest))?,
        "export-request" => serde_json::to_value(schema_for!(ExportRequest))?,
        other => anyhow::bail!("Unsupported API schema: {other}"),
    };
    Ok(schema)
}
