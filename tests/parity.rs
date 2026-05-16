use mtgdeckbuilder::eval::consistency::score_consistency;
use mtgdeckbuilder::export::arena::format_arena_decklist;
use mtgdeckbuilder::features::deck_features::extract_deck_features;
use mtgdeckbuilder::ingest::card_database::CardDatabase;
use mtgdeckbuilder::ingest::collection_parser::{inspect_collection_schema, parse_collection_csv};
use mtgdeckbuilder::ingest::decklist_importer::{
    parse_arena_decklist_file, parse_arena_decklist_text,
};
use mtgdeckbuilder::mcp::server::SAFE_TOOLS;
use mtgdeckbuilder::rules::validator::DeckValidator;
use mtgdeckbuilder::sim::early_turns::simulate_first_three_turns;
use mtgdeckbuilder::sim::opening_hand::simulate_opening_hands;
use pretty_assertions::assert_eq;
use serde_json::json;

const CARDS: &str = "tests/fixtures/cards_scryfall.json";
const COLLECTION: &str = "tests/fixtures/collection.csv";
const DECK: &str = "examples/sample_deck.txt";

#[test]
fn parses_main_sideboard_companion_and_strips_arena_suffix() {
    let deck = parse_arena_decklist_text(
        r#"
Companion
1 Test Companion (TST) 1

Deck
4 Test Bear (TST) 2
24 Forest

Sideboard
3 Test Shield
"#,
    )
    .unwrap();

    assert_eq!(deck.mainboard.get("Test Bear"), Some(&4));
    assert_eq!(deck.mainboard.get("Forest"), Some(&24));
    assert_eq!(deck.sideboard.get("Test Shield"), Some(&3));
    assert_eq!(deck.companion.as_deref(), Some("Test Companion"));
}

#[test]
fn rejects_invalid_deck_count_lines() {
    let err = parse_arena_decklist_text("Deck\nNo count here\n").unwrap_err();
    assert!(err.to_string().contains("expected '<count> <card name>'"));
}

#[test]
fn collection_schema_and_owned_counts_match_fixture() {
    let schema = inspect_collection_schema(COLLECTION).unwrap();
    assert_eq!(schema.name_field.as_deref(), Some("Name"));
    assert_eq!(schema.count_field.as_deref(), Some("Quantity"));
    assert_eq!(
        schema.fields,
        vec!["Name".to_string(), "Quantity".to_string()]
    );
    assert_eq!(schema.sample_rows.len(), 3);

    let collection = parse_collection_csv(COLLECTION, None, None).unwrap();
    assert_eq!(collection.owned("Test Bear"), 4);
    assert_eq!(collection.owned("test bear"), 4);
}

#[test]
fn collection_parser_records_warnings_for_bad_rows() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.csv");
    std::fs::write(
        &path,
        "Name,Quantity\nTest Bear,2.0\n,-1\nTest Trick,nope\nTest Hydra,-3\n",
    )
    .unwrap();

    let collection = parse_collection_csv(&path, None, None).unwrap();
    assert_eq!(collection.owned("Test Bear"), 2);
    assert_eq!(collection.owned("Test Trick"), 0);
    assert_eq!(collection.owned("Test Hydra"), 0);
    assert!(
        collection
            .warnings
            .iter()
            .any(|w| w.contains("empty card name"))
    );
    assert!(
        collection
            .warnings
            .iter()
            .any(|w| w.contains("nonnumeric count"))
    );
    assert!(
        collection
            .warnings
            .iter()
            .any(|w| w.contains("negative count"))
    );
}

#[test]
fn sample_deck_validation_matches_fixture_golden() {
    let db = CardDatabase::from_scryfall_file(CARDS).unwrap();
    let collection = parse_collection_csv(COLLECTION, None, None).unwrap();
    let deck = parse_arena_decklist_file(DECK).unwrap();
    let report = DeckValidator::new(db).validate(&deck, "standard", Some(&collection), false, None);

    assert!(report.ok());
    assert_eq!(
        serde_json::to_value(report).unwrap(),
        json!({
          "assumptions": [],
          "format_name": "standard",
          "issues": [],
          "main_count": 60,
          "sideboard_count": 15,
          "status": "PASS",
          "wildcards_required": {}
        })
    );
}

#[test]
fn copy_limit_failure_matches_fixture_golden() {
    let db = CardDatabase::from_scryfall_file(CARDS).unwrap();
    let deck = parse_arena_decklist_text("Deck\n5 Test Bear\n55 Forest\n").unwrap();
    let report = DeckValidator::new(db).validate(&deck, "standard", None, false, None);

    assert!(!report.ok());
    assert_eq!(
        serde_json::to_value(report).unwrap(),
        json!({
          "assumptions": [],
          "format_name": "standard",
          "issues": [
            {
              "card_name": "Test Bear",
              "code": "copy_limit.exceeded",
              "details": {
                "actual": 5,
                "limit": 4
              },
              "message": "Test Bear has 5 copies; limit is 4.",
              "severity": "ERROR"
            }
          ],
          "main_count": 60,
          "sideboard_count": 0,
          "status": "FAIL",
          "wildcards_required": {}
        })
    );
}

#[test]
fn arena_export_preserves_fixture_order() {
    let deck = parse_arena_decklist_file(DECK).unwrap();
    assert_eq!(
        format_arena_decklist(&deck),
        "Deck\n24 Forest\n4 Test Bear\n4 Test Ranger\n4 Test Trick\n4 Test Growth\n4 Test Removal\n4 Test Engine\n4 Test Hydra\n4 Test Scout\n4 Test Sentinel\n\nSideboard\n3 Test Shield\n3 Test Naturalize\n3 Test Grave Hate\n3 Test Sweeper Guard\n3 Test Control Plan\n"
    );
}

#[test]
fn deck_features_match_fixture_golden() {
    let db = CardDatabase::from_scryfall_file(CARDS).unwrap();
    let deck = parse_arena_decklist_file(DECK).unwrap();
    assert_eq!(
        extract_deck_features(&deck, &db),
        json!({
          "average_nonland_mana_value": 1.778,
          "deck_size": 60,
          "interaction_density": 0.06666666666666667,
          "land_count": 24,
          "land_ratio": 0.4,
          "mana_curve": {
            "1": 16,
            "2": 12,
            "3": 8
          },
          "nonland_count": 36,
          "protection_density": 0.13333333333333333,
          "role_counts": {
            "interaction": 4,
            "land": 24,
            "landfall": 12,
            "protection": 8,
            "selection_or_card_advantage": 12,
            "threat": 28
          },
          "source_counts": {
            "B": 0,
            "G": 24,
            "R": 0,
            "U": 0,
            "W": 0
          },
          "threat_density": 0.4666666666666667
        })
    );
}

#[test]
fn seeded_simulations_use_stable_chacha_metrics() {
    let db = CardDatabase::from_scryfall_file(CARDS).unwrap();
    let deck = parse_arena_decklist_file(DECK).unwrap();

    let opening = simulate_opening_hands(&deck, &db, "arena_n2", 50, 42, 2).unwrap();
    let opening_repeat = simulate_opening_hands(&deck, &db, "arena_n2", 50, 42, 2).unwrap();
    assert_eq!(opening.metrics, opening_repeat.metrics);
    assert_eq!(
        serde_json::to_value(&opening).unwrap(),
        json!({
          "assumptions": [
            "Opening-hand quality is not match win rate.",
            "Arena-like approximation; exact MTG Arena Bo1 hand smoothing is not public."
          ],
          "metrics": {
            "flood_risk_opening_rate": 0.0,
            "has_threat_rate": 1.0,
            "keepable_7_rate": 0.98,
            "low_quality_forced_keep_rate": 0.0,
            "mulligan_to_5_rate": 0.0,
            "mulligan_to_6_rate": 0.02,
            "no_primary_source_rate": 0.0,
            "screw_risk_opening_rate": 0.06,
            "turn_1_or_2_play_rate": 1.0
          },
          "mode": "arena_n2",
          "seed": 42,
          "trials": 50
        })
    );

    let early = simulate_first_three_turns(&deck, &db, 50, 42).unwrap();
    let early_repeat = simulate_first_three_turns(&deck, &db, 50, 42).unwrap();
    assert_eq!(early.metrics, early_repeat.metrics);
    assert_eq!(
        serde_json::to_value(&early).unwrap(),
        json!({
          "assumptions": [
            "Deterministic sequencing heuristic; not a gameplay simulator and not a match win-rate model."
          ],
          "metrics": {
            "did_nothing_by_turn_3_rate": 0.0,
            "missed_land_drop_before_turn_3_rate": 0.16,
            "threat_by_turn_3_rate": 0.82,
            "turn_2_plan_online_rate": 0.98,
            "turn_3_plan_online_rate": 0.98
          },
          "seed": 42,
          "trials": 50
        })
    );

    assert_eq!(
        score_consistency(&opening, &early),
        json!({"consistency_score": 0.96})
    );
}

#[test]
fn safe_tool_surface_excludes_arena_client_control() {
    assert!(SAFE_TOOLS.contains(&"validate_deck"));
    assert!(SAFE_TOOLS.contains(&"export_arena_decklist"));
    assert!(!SAFE_TOOLS.iter().any(|tool| tool.contains("arena_client")));
    assert!(!SAFE_TOOLS.iter().any(|tool| tool.contains("screen")));
}
