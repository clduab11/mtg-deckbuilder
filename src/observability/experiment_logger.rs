use anyhow::Result;
use chrono::Utc;
use serde_json::{Map, Value};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ExperimentRecord {
    pub experiment_id: String,
    pub created_at: String,
    pub payload: Value,
}

impl ExperimentRecord {
    pub fn to_json(&self) -> Result<String> {
        let mut object = Map::new();
        object.insert(
            "created_at".to_string(),
            Value::String(self.created_at.clone()),
        );
        object.insert(
            "experiment_id".to_string(),
            Value::String(self.experiment_id.clone()),
        );
        if let Value::Object(payload) = &self.payload {
            for (key, value) in payload {
                object.insert(key.clone(), value.clone());
            }
        }
        Ok(serde_json::to_string(&Value::Object(object))?)
    }
}

pub fn new_experiment_id(prefix: &str) -> String {
    format!("{}_{}", prefix, Utc::now().format("%Y%m%dT%H%M%SZ"))
}

pub fn write_experiment(
    path: impl AsRef<Path>,
    payload: Value,
    experiment_id: Option<&str>,
) -> Result<ExperimentRecord> {
    let record = ExperimentRecord {
        experiment_id: experiment_id
            .map(str::to_string)
            .unwrap_or_else(|| new_experiment_id("exp")),
        created_at: Utc::now().to_rfc3339(),
        payload,
    };
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    use std::io::Write;
    writeln!(file, "{}", record.to_json()?)?;
    Ok(record)
}
