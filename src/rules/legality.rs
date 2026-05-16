use crate::core::models::Card;

fn normalized_key(value: &str) -> String {
    value.to_lowercase().trim().replace(' ', "")
}

pub fn legality_status(card: &Card, format_name: &str) -> String {
    let key = normalized_key(format_name);
    for (format, status) in &card.legalities {
        if normalized_key(format) == key {
            return status.to_lowercase();
        }
    }
    "unknown".to_string()
}

pub fn is_legal(card: &Card, format_name: &str) -> bool {
    matches!(
        legality_status(card, format_name).as_str(),
        "legal" | "restricted"
    )
}

pub fn is_banned(card: &Card, format_name: &str) -> bool {
    legality_status(card, format_name) == "banned"
}

pub fn is_restricted(card: &Card, format_name: &str) -> bool {
    legality_status(card, format_name) == "restricted"
}
