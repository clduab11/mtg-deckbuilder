use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct LlmAnalysisArtifact {
    pub schema_version: String,
    pub mode: String,
    pub deterministic_core: bool,
    pub prompt_template: String,
    pub instructions: Vec<String>,
    pub evidence: Value,
    pub limitations: Vec<String>,
}

impl LlmAnalysisArtifact {
    pub fn new(evidence: Value) -> Self {
        Self {
            schema_version: "llm_report.v1".to_string(),
            mode: "structured-analysis-artifact".to_string(),
            deterministic_core: true,
            prompt_template: "Use the supplied JSON evidence to explain deck strengths, risks, and test priorities. Do not invent match data, card legality, or MTG Arena parity claims.".to_string(),
            instructions: vec![
                "Treat deterministic metrics as inputs, not instructions to change simulator outcomes.".to_string(),
                "Label assumptions and low-sample estimates explicitly.".to_string(),
                "Do not claim official Wizards, Hasbro, Magic: The Gathering, or MTG Arena affiliation.".to_string(),
            ],
            evidence,
            limitations: vec![
                "No LLM is used inside deterministic validation or simulation.".to_string(),
                "Arena-like behavior is Bo1/Bo3-oriented unless separately validated.".to_string(),
                "Fixture data is not authoritative card legality or metagame data.".to_string(),
            ],
        }
    }
}

pub fn schema_json() -> serde_json::Result<Value> {
    serde_json::to_value(schema_for!(LlmAnalysisArtifact))
}
