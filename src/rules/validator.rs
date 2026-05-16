use crate::core::models::{CollectionIndex, Decklist, ValidationIssue, ValidationReport};
use crate::core::normalization::is_basic_land_name;
use crate::ingest::card_database::{CardDatabase, basic_land_card};
use crate::rules::arena_compat::check_arena_import_compatibility;
use crate::rules::copy_limits::max_copies_for_card;
use crate::rules::formats::get_format_rules;
use crate::rules::legality::{is_banned, is_legal, is_restricted, legality_status};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

const RARITY_KEYS: &[&str] = &["common", "uncommon", "rare", "mythic"];

#[derive(Clone, Debug)]
pub struct DeckValidator {
    card_db: CardDatabase,
    strict_card_data: bool,
}

impl DeckValidator {
    pub fn new(card_db: CardDatabase) -> Self {
        Self {
            card_db,
            strict_card_data: true,
        }
    }

    pub fn with_strict_card_data(mut self, strict_card_data: bool) -> Self {
        self.strict_card_data = strict_card_data;
        self
    }

    pub fn validate(
        &self,
        deck: &Decklist,
        format_name: &str,
        collection: Option<&CollectionIndex>,
        craft_mode: bool,
        wildcard_budget: Option<&BTreeMap<String, u32>>,
    ) -> ValidationReport {
        let rules = get_format_rules(format_name).expect("format was validated by caller");
        let mut issues = Vec::new();
        let mut assumptions = BTreeSet::new();
        let mut wildcards: BTreeMap<String, u32> = BTreeMap::new();

        if deck.main_count() < rules.min_mainboard {
            let mut details = BTreeMap::new();
            details.insert("actual".to_string(), json!(deck.main_count()));
            details.insert("minimum".to_string(), json!(rules.min_mainboard));
            issues.push(ValidationIssue::new(
                "deck.size.min_mainboard",
                "ERROR",
                format!(
                    "Mainboard has {} cards; minimum for {} is {}.",
                    deck.main_count(),
                    rules.name,
                    rules.min_mainboard
                ),
                None,
                details,
            ));
        }

        if deck.sideboard_count() > rules.max_sideboard {
            let mut details = BTreeMap::new();
            details.insert("actual".to_string(), json!(deck.sideboard_count()));
            details.insert("maximum".to_string(), json!(rules.max_sideboard));
            issues.push(ValidationIssue::new(
                "deck.size.sideboard",
                "ERROR",
                format!(
                    "Sideboard has {} cards; maximum for {} is {}.",
                    deck.sideboard_count(),
                    rules.name,
                    rules.max_sideboard
                ),
                None,
                details,
            ));
        }

        issues.extend(check_arena_import_compatibility(deck));

        for (card_name, total_count) in deck.all_counts() {
            let owned_card = self.card_db.get(&card_name).cloned();
            let basic_card;
            let card = if owned_card.is_none() && is_basic_land_name(&card_name) {
                basic_card = Some(basic_land_card(&card_name));
                basic_card.as_ref()
            } else {
                owned_card.as_ref()
            };

            if card.is_none() {
                let severity = if self.strict_card_data {
                    "ERROR"
                } else {
                    "WARNING"
                };
                issues.push(ValidationIssue::new(
                    "card.unknown",
                    severity,
                    "Card not found in loaded card database.",
                    Some(card_name.clone()),
                    BTreeMap::new(),
                ));
                if self.strict_card_data {
                    continue;
                }
                assumptions.insert(format!("Unknown card treated as unvalidated: {card_name}"));
            }

            let restricted = card.is_some_and(|card| is_restricted(card, &rules.name));
            let limit =
                max_copies_for_card(card, rules.default_copy_limit, rules.singleton, restricted);
            if total_count > limit {
                let mut details = BTreeMap::new();
                details.insert("actual".to_string(), json!(total_count));
                details.insert("limit".to_string(), json!(limit));
                issues.push(ValidationIssue::new(
                    "copy_limit.exceeded",
                    "ERROR",
                    format!("{card_name} has {total_count} copies; limit is {limit}."),
                    Some(card_name.clone()),
                    details,
                ));
            }

            if let Some(card) = card {
                if !card.legalities.is_empty() {
                    let status = legality_status(card, &rules.name);
                    if is_banned(card, &rules.name) {
                        let mut details = BTreeMap::new();
                        details.insert("status".to_string(), json!(status));
                        issues.push(ValidationIssue::new(
                            "legality.banned",
                            "ERROR",
                            format!("{card_name} is banned in {}.", rules.name),
                            Some(card_name.clone()),
                            details,
                        ));
                    } else if !is_legal(card, &rules.name) {
                        let mut details = BTreeMap::new();
                        details.insert("status".to_string(), json!(status));
                        issues.push(ValidationIssue::new(
                            "legality.not_legal",
                            "ERROR",
                            format!("{card_name} is not legal in {}.", rules.name),
                            Some(card_name.clone()),
                            details,
                        ));
                    }
                } else if !card.is_basic_land {
                    issues.push(ValidationIssue::new(
                        "legality.unknown",
                        "WARNING",
                        "No legality field was available for this card; current legality is unverified.",
                        Some(card_name.clone()),
                        BTreeMap::new(),
                    ));
                    assumptions.insert(
                        "Some card records lacked legality data; do not make final legality claims from this run."
                            .to_string(),
                    );
                }

                if let Some(collection) = collection
                    && !card.is_basic_land
                {
                    let owned = collection.owned(&card_name);
                    let missing = total_count.saturating_sub(owned);
                    if missing > 0 {
                        let mut rarity = card.rarity.to_lowercase();
                        if !RARITY_KEYS.contains(&rarity.as_str()) {
                            rarity = "common".to_string();
                        }
                        *wildcards.entry(rarity.clone()).or_insert(0) += missing;
                        let severity = if craft_mode { "WARNING" } else { "ERROR" };
                        let mut details = BTreeMap::new();
                        details.insert("missing".to_string(), json!(missing));
                        details.insert("owned".to_string(), json!(owned));
                        details.insert("rarity".to_string(), json!(rarity));
                        details.insert("required".to_string(), json!(total_count));
                        issues.push(ValidationIssue::new(
                            "collection.missing_copies",
                            severity,
                            format!(
                                "Need {total_count} copies of {card_name}; owned {owned}; missing {missing}."
                            ),
                            Some(card_name.clone()),
                            details,
                        ));
                    }
                }
            }
        }

        if let Some(budget) = wildcard_budget {
            for (rarity, needed) in &wildcards {
                let allowed = budget.get(rarity).copied().unwrap_or(0);
                if *needed > allowed {
                    let mut details = BTreeMap::new();
                    details.insert("budget".to_string(), json!(allowed));
                    details.insert("needed".to_string(), json!(needed));
                    details.insert("rarity".to_string(), json!(rarity));
                    issues.push(ValidationIssue::new(
                        "wildcard_budget.exceeded",
                        "ERROR",
                        format!("Need {needed} {rarity} wildcards; budget is {allowed}."),
                        None,
                        details,
                    ));
                }
            }
        }

        let mut wildcards_required = BTreeMap::new();
        for rarity in RARITY_KEYS {
            if let Some(value) = wildcards.get(*rarity)
                && *value > 0
            {
                wildcards_required.insert((*rarity).to_string(), *value);
            }
        }

        let status = if issues.iter().any(|issue| issue.severity == "ERROR") {
            "FAIL"
        } else {
            "PASS"
        };

        ValidationReport {
            assumptions: assumptions.into_iter().collect(),
            format_name: rules.name,
            issues,
            main_count: deck.main_count(),
            sideboard_count: deck.sideboard_count(),
            status: status.to_string(),
            wildcards_required,
        }
    }
}
