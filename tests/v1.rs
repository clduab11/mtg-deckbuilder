use mtgdeckbuilder::api_contract::{DeckValidateRequest, ROUTES};
use mtgdeckbuilder::catalog::load_catalog;
use mtgdeckbuilder::llm::LlmAnalysisArtifact;
use mtgdeckbuilder::sim::bo1::Bo1Config;
use mtgdeckbuilder::sim::bo3::Bo3Config;
use mtgdeckbuilder::stats::{
    DraftPickRecord, GameRecord, rate, summarize_constructed, summarize_draft_card,
};
use serde_json::json;

#[test]
fn catalog_loads_csv_json_jsonl_and_yaml() {
    for (path, format, expected_count) in [
        ("examples/sample_catalog.csv", "csv", 3),
        ("examples/sample_catalog.json", "json", 2),
        ("examples/sample_catalog.jsonl", "jsonl", 2),
        ("examples/sample_catalog.yaml", "yaml", 2),
    ] {
        let loaded = load_catalog(path, Some(format)).unwrap();
        assert_eq!(loaded.report.card_count, expected_count);
        assert!(loaded.database.get("Forest").is_some());
    }
}

#[test]
fn bo_configs_state_oriented_assumptions() {
    let bo1 = Bo1Config::default();
    assert_eq!(bo1.sideboard_slots_available, 7);
    assert!(
        bo1.assumptions
            .iter()
            .any(|a| a.contains("not exact MTG Arena parity"))
    );

    let bo3 = Bo3Config::default();
    assert_eq!(bo3.sideboard_slots_available, 15);
    assert_eq!(bo3.games_per_match, 3);
}

#[test]
fn constructed_stats_include_confidence_and_warnings() {
    let summary = summarize_constructed(&[
        GameRecord {
            match_id: "m1".to_string(),
            game_number: 1,
            queue: "bo1".to_string(),
            format_name: "standard".to_string(),
            opponent_archetype: Some("aggro".to_string()),
            won: true,
            mulligans: 0,
            sideboarded: false,
        },
        GameRecord {
            match_id: "m2".to_string(),
            game_number: 1,
            queue: "bo3".to_string(),
            format_name: "standard".to_string(),
            opponent_archetype: Some("control".to_string()),
            won: false,
            mulligans: 1,
            sideboarded: true,
        },
    ]);
    assert_eq!(summary.games, 2);
    assert_eq!(summary.game_win_rate.successes, 1);
    assert!(summary.game_win_rate.sample_size_warning.is_some());
    assert!(summary.matchup_matrix.contains_key("aggro"));

    let larger = rate(75, 100);
    assert_eq!(larger.reliability, "high");
    assert!(larger.interval.lower < larger.interval.upper);
}

#[test]
fn draft_stats_include_pick_and_drawn_metrics() {
    let summary = summarize_draft_card(
        "Test Bear",
        &[
            DraftPickRecord {
                draft_id: "d1".to_string(),
                card_name: "Test Bear".to_string(),
                pack_number: 1,
                pick_number: 2,
                seen_at_pick: 3,
                taken: true,
                opening_hand_games: 3,
                opening_hand_wins: 2,
                drawn_games: 5,
                drawn_wins: 4,
                games: 8,
                wins: 5,
                trophies: 1,
                events: 2,
                color_pair: Some("G".to_string()),
                archetype: Some("stompy".to_string()),
                wheeled: false,
            },
            DraftPickRecord {
                draft_id: "d2".to_string(),
                card_name: "Test Bear".to_string(),
                pack_number: 1,
                pick_number: 8,
                seen_at_pick: 9,
                taken: false,
                opening_hand_games: 1,
                opening_hand_wins: 0,
                drawn_games: 2,
                drawn_wins: 1,
                games: 3,
                wins: 1,
                trophies: 0,
                events: 1,
                color_pair: Some("G".to_string()),
                archetype: Some("stompy".to_string()),
                wheeled: true,
            },
        ],
    );
    assert_eq!(summary.card_name, "Test Bear");
    assert_eq!(summary.average_last_seen_at, 6.0);
    assert!(summary.game_in_hand_win_rate.rate > 0.0);
    assert!(summary.wheel_rate.rate > 0.0);
}

#[test]
fn llm_artifact_is_structured_and_non_controlling() {
    let artifact = LlmAnalysisArtifact::new(json!({"validation": {"status": "PASS"}}));
    assert_eq!(artifact.schema_version, "llm_report.v1");
    assert!(artifact.deterministic_core);
    assert!(
        artifact.instructions.iter().any(
            |instruction| instruction.contains("not instructions to change simulator outcomes")
        )
    );
}

#[test]
fn api_contract_routes_and_serialization_are_backend_ready() {
    assert!(ROUTES.contains(&("POST", "/deck/validate")));
    assert!(ROUTES.contains(&("GET", "/simulation/{id}/report")));
    let request = DeckValidateRequest {
        deck_text: "Deck\n24 Forest\n".to_string(),
        catalog_path: "examples/sample_catalog.json".to_string(),
        format_name: "standard".to_string(),
    };
    let value = serde_json::to_value(request).unwrap();
    assert_eq!(value["format_name"], "standard");
}
