use crate::core::models::Decklist;
use crate::core::normalization::strip_arena_set_suffix;
use anyhow::{Result, anyhow};
use regex::Regex;
use std::path::Path;

fn add_count(zone: &mut indexmap::IndexMap<String, u32>, name: String, count: u32) {
    *zone.entry(name).or_insert(0) += count;
}

pub fn parse_arena_decklist_text(text: &str) -> Result<Decklist> {
    parse_arena_decklist_text_with_source(text, None)
}

pub fn parse_arena_decklist_text_with_source(
    text: &str,
    source_path: Option<String>,
) -> Result<Decklist> {
    let count_line = Regex::new(r"^\s*(\d+)\s+(.+?)\s*$").unwrap();
    let mut section = "main";
    let mut deck = Decklist {
        source_path,
        ..Decklist::default()
    };

    for (idx, raw_line) in text.lines().enumerate() {
        let line_number = idx + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let lowered = line.to_lowercase();
        match lowered.as_str() {
            "deck" | "main" | "mainboard" => {
                section = "main";
                continue;
            }
            "sideboard" => {
                section = "sideboard";
                continue;
            }
            "companion" => {
                section = "companion";
                continue;
            }
            _ => {}
        }

        let captures = count_line.captures(line).ok_or_else(|| {
            anyhow!("Line {line_number}: expected '<count> <card name>', got {line:?}")
        })?;
        let count: u32 = captures[1].parse()?;
        if count == 0 {
            return Err(anyhow!("Line {line_number}: count must be positive"));
        }
        let name = strip_arena_set_suffix(&captures[2]);
        if name.is_empty() {
            return Err(anyhow!("Line {line_number}: empty card name"));
        }

        match section {
            "sideboard" => add_count(&mut deck.sideboard, name, count),
            "companion" => {
                if count != 1 {
                    return Err(anyhow!("Companion section must contain exactly one copy"));
                }
                deck.companion = Some(name);
            }
            _ => add_count(&mut deck.mainboard, name, count),
        }
    }

    Ok(deck)
}

pub fn parse_arena_decklist_file(path: impl AsRef<Path>) -> Result<Decklist> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path)?;
    parse_arena_decklist_text_with_source(&text, Some(path.to_string_lossy().to_string()))
}
