use crate::core::models::Decklist;
use anyhow::{Result, anyhow};

pub fn swap_mainboard_card(
    deck: &Decklist,
    remove_name: &str,
    add_name: &str,
    count: u32,
) -> Result<Decklist> {
    if count == 0 {
        return Err(anyhow!("count must be positive"));
    }
    let mut next = deck.clone();
    let existing = next.mainboard.get(remove_name).copied().unwrap_or(0);
    if existing < count {
        return Err(anyhow!(
            "Cannot remove {count} copies of {remove_name}; only {existing} present"
        ));
    }
    if existing == count {
        next.mainboard.shift_remove(remove_name);
    } else if let Some(slot) = next.mainboard.get_mut(remove_name) {
        *slot -= count;
    }
    *next.mainboard.entry(add_name.to_string()).or_insert(0) += count;
    Ok(next)
}
