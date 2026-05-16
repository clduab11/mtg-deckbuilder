use crate::sim::early_turns::EarlyTurnSimulationResult;
use crate::sim::opening_hand::OpeningHandSimulationResult;
use serde_json::{Value, json};

fn metric(value: &serde_json::Map<String, Value>, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

pub fn score_consistency(
    opening: &OpeningHandSimulationResult,
    early: &EarlyTurnSimulationResult,
) -> Value {
    let m = opening.metrics.as_object().expect("metrics object");
    let e = early.metrics.as_object().expect("metrics object");
    let score = 0.25 * metric(m, "keepable_7_rate")
        + 0.15 * (1.0 - metric(m, "screw_risk_opening_rate"))
        + 0.10 * (1.0 - metric(m, "flood_risk_opening_rate"))
        + 0.20 * metric(e, "turn_2_plan_online_rate")
        + 0.20 * metric(e, "turn_3_plan_online_rate")
        + 0.10 * metric(e, "threat_by_turn_3_rate");
    json!({"consistency_score": round4(score)})
}
