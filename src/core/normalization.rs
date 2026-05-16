use regex::Regex;

pub fn normalize_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .replace("//", "/")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn is_basic_land_name(name: &str) -> bool {
    matches!(
        normalize_name(name).as_str(),
        "plains" | "island" | "swamp" | "mountain" | "forest" | "wastes"
    )
}

pub fn strip_arena_set_suffix(card_text: &str) -> String {
    let text = card_text.trim();
    let re = Regex::new(r"\s+\([A-Z0-9_]+\)\s+\d+[a-zA-Z]?(?:\s+\*F\*)?$").unwrap();
    re.replace(text, "").trim().to_string()
}
