use crate::core::models::Card;

pub fn max_copies_for_card(
    card: Option<&Card>,
    default_limit: u32,
    singleton: bool,
    restricted: bool,
) -> u32 {
    if card.is_some_and(|card| card.is_basic_land) {
        return 10_000;
    }
    if restricted || singleton {
        return 1;
    }
    default_limit
}
