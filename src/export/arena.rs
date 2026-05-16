use crate::core::models::Decklist;

pub fn format_arena_decklist(deck: &Decklist) -> String {
    let mut lines = Vec::new();
    if let Some(companion) = &deck.companion {
        lines.push("Companion".to_string());
        lines.push(format!("1 {companion}"));
        lines.push(String::new());
    }
    lines.push("Deck".to_string());
    for (name, count) in &deck.mainboard {
        lines.push(format!("{count} {name}"));
    }
    if !deck.sideboard.is_empty() {
        lines.push(String::new());
        lines.push("Sideboard".to_string());
        for (name, count) in &deck.sideboard {
            lines.push(format!("{count} {name}"));
        }
    }
    format!("{}\n", lines.join("\n").trim_end())
}
