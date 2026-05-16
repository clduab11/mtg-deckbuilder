use crate::core::models::{Decklist, ValidationReport};
use crate::export::arena::format_arena_decklist;

pub fn render_validation_report(deck: &Decklist, report: &ValidationReport) -> String {
    let mut lines = vec![
        "# Deck Validation Report".to_string(),
        String::new(),
        format!("- Status: {}", report.status),
        format!("- Format: {}", report.format_name),
        format!("- Mainboard: {}", report.main_count),
        format!("- Sideboard: {}", report.sideboard_count),
        format!("- Wildcards required: {:?}", report.wildcards_required),
        String::new(),
        "## Issues".to_string(),
    ];
    if report.issues.is_empty() {
        lines.push("- None".to_string());
    } else {
        for issue in &report.issues {
            let card = issue
                .card_name
                .as_ref()
                .map(|name| format!(" [{name}]"))
                .unwrap_or_default();
            lines.push(format!(
                "- {} {}{}: {}",
                issue.severity, issue.code, card, issue.message
            ));
        }
    }
    if !report.assumptions.is_empty() {
        lines.extend([String::new(), "## Assumptions".to_string()]);
        for assumption in &report.assumptions {
            lines.push(format!("- {assumption}"));
        }
    }
    lines.extend([
        String::new(),
        "## Arena Export".to_string(),
        String::new(),
        "```".to_string(),
        format_arena_decklist(deck).trim_end().to_string(),
        "```".to_string(),
    ]);
    format!("{}\n", lines.join("\n"))
}
