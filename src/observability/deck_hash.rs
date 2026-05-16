use crate::core::models::Decklist;
use crate::export::arena::format_arena_decklist;
use sha2::{Digest, Sha256};

pub fn deck_hash(deck: &Decklist) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format_arena_decklist(deck).as_bytes());
    let digest = hasher.finalize();
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
        .chars()
        .take(16)
        .collect()
}
