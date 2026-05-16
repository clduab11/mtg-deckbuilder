use crate::core::models::Decklist;
use crate::ingest::card_database::CardDatabase;
use std::collections::BTreeMap;

pub fn mana_curve(
    deck: &Decklist,
    card_db: &CardDatabase,
    max_bucket: u32,
) -> BTreeMap<String, u32> {
    let mut raw: BTreeMap<u32, u32> = BTreeMap::new();
    let mut top_bucket = 0;
    for (name, count) in &deck.mainboard {
        let Some(card) = card_db.get(name) else {
            continue;
        };
        if card.is_land() {
            continue;
        }
        let mv = card.mana_value as u32;
        let bucket = if mv >= max_bucket { max_bucket } else { mv };
        if bucket == max_bucket {
            top_bucket += count;
        } else {
            *raw.entry(bucket).or_insert(0) += count;
        }
    }

    let mut curve = BTreeMap::new();
    for (bucket, count) in raw {
        curve.insert(bucket.to_string(), count);
    }
    if top_bucket > 0 {
        curve.insert(format!("{max_bucket}+"), top_bucket);
    }
    curve
}
