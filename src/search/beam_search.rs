use crate::core::models::Decklist;

pub fn beam_search_seed(seed: &Decklist) -> Vec<Decklist> {
    vec![seed.clone()]
}
