pub const POSTURES: &[&str] = &[
    "favorable",
    "slightly favorable",
    "even",
    "slightly unfavorable",
    "unfavorable",
    "hostile",
];

pub fn classify_matchup_posture(score: f64) -> &'static str {
    if score >= 0.75 {
        "favorable"
    } else if score >= 0.60 {
        "slightly favorable"
    } else if score >= 0.45 {
        "even"
    } else if score >= 0.30 {
        "slightly unfavorable"
    } else if score >= 0.15 {
        "unfavorable"
    } else {
        "hostile"
    }
}
