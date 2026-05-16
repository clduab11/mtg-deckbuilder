use std::collections::BTreeMap;

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

pub fn bo1_ranked_score(
    metrics: &BTreeMap<String, f64>,
    matchup_posture_score: f64,
    wildcard_efficiency: f64,
) -> f64 {
    let get = |key: &str| metrics.get(key).copied().unwrap_or(0.0);
    round4(
        0.20 * get("keepable_7_rate")
            + 0.15 * get("turn_2_plan_online_rate")
            + 0.15 * get("turn_3_plan_online_rate")
            + 0.10 * (1.0 - get("no_primary_source_rate"))
            + 0.10 * matchup_posture_score
            + 0.10 * get("has_threat_rate")
            + 0.08 * get("protection_or_interaction_rate")
            + 0.07 * (1.0 - get("dead_card_opening_rate"))
            + 0.05 * wildcard_efficiency,
    )
}
