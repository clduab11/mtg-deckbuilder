use crate::core::models::Card;
use crate::rules::legality::{is_banned, is_restricted};

pub fn ban_status(card: &Card, format_name: &str) -> &'static str {
    if is_banned(card, format_name) {
        "banned"
    } else if is_restricted(card, format_name) {
        "restricted"
    } else {
        "not_banned"
    }
}
