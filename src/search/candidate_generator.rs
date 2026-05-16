use crate::core::models::Decklist;

pub fn identity_candidate(deck: &Decklist) -> Decklist {
    deck.clone()
}
