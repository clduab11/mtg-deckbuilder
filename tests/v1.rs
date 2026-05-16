use mtgdeckbuilder::api_contract::{DeckValidateRequest, ROUTES};
use mtgdeckbuilder::catalog::load_catalog;
use mtgdeckbuilder::llm::LlmAnalysisArtifact;
use mtgdeckbuilder::result_log::{load_result_log, schema_json, summarize_loaded_result_log};
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
fn catalog_csv_maps_optional_card_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.csv");
    std::fs::write(
        &path,
        "Name,Type,Keywords,Games,Legalities,Arena ID,Digital,Rebalanced,standard\n\
         Test Spell,Instant,Double strike;Flying,\"arena,paper\",explorer:legal;historic:not_legal,123,true,yes,legal\n",
    )
    .unwrap();

    let loaded = load_catalog(&path, Some("csv")).unwrap();
    let card = loaded.database.get("Test Spell").unwrap();
    assert_eq!(card.keywords, vec!["Double strike", "Flying"]);
    assert_eq!(card.games, vec!["arena", "paper"]);
    assert_eq!(
        card.legalities.get("explorer").map(String::as_str),
        Some("legal")
    );
    assert_eq!(
        card.legalities.get("historic").map(String::as_str),
        Some("not_legal")
    );
    assert_eq!(
        card.legalities.get("standard").map(String::as_str),
        Some("legal")
    );
    assert_eq!(card.arena_id, Some(123));
    assert!(card.is_digital);
    assert!(card.is_rebalanced);
}

#[test]
fn catalog_rejects_unknown_json_and_yaml_schema_versions() {
    let dir = tempfile::tempdir().unwrap();
    let json_path = dir.path().join("catalog.json");
    std::fs::write(
        &json_path,
        r#"{"schema_version":"catalog.v2","cards":[{"name":"Test Spell"}]}"#,
    )
    .unwrap();
    let json_error = load_catalog(&json_path, Some("json")).unwrap_err();
    assert!(
        json_error
            .to_string()
            .contains("Unsupported catalog schema_version")
    );

    let yaml_path = dir.path().join("catalog.yaml");
    std::fs::write(
        &yaml_path,
        "schema_version: catalog.v2\ncards:\n  - name: Test Spell\n",
    )
    .unwrap();
    let yaml_error = load_catalog(&yaml_path, Some("yaml")).unwrap_err();
    assert!(
        yaml_error
            .to_string()
            .contains("Unsupported catalog schema_version")
    );
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
            play_draw: Some("play".to_string()),
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
            play_draw: Some("draw".to_string()),
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
    assert!(summary.play_draw_performance.contains_key("play"));

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
fn result_log_loads_csv_json_and_jsonl() {
    for (path, format, expected_games, expected_draft_picks) in [
        ("tests/fixtures/result_logs.csv", "csv", 3, 1),
        ("tests/fixtures/result_logs.json", "json", 3, 2),
        ("tests/fixtures/result_logs.jsonl", "jsonl", 3, 1),
    ] {
        let loaded = load_result_log(path, Some(format)).unwrap();
        assert_eq!(loaded.report.schema_version, "result-log-load-report.v1");
        assert_eq!(loaded.report.game_count, expected_games);
        assert_eq!(loaded.report.draft_pick_count, expected_draft_picks);
        assert_eq!(loaded.games.len(), expected_games);
        assert_eq!(loaded.draft_picks.len(), expected_draft_picks);
        assert_eq!(loaded.games[0].play_draw.as_deref(), Some("play"));
        assert_eq!(loaded.games[0].opponent_archetype.as_deref(), Some("aggro"));
        assert_eq!(loaded.games[1].mulligans, 1);
        assert!(loaded.games[1].sideboarded);
        assert_eq!(loaded.draft_picks[0].pack_number, 1);
        assert_eq!(loaded.draft_picks[0].pick_number, 2);
    }
}

#[test]
fn result_log_rejects_bad_schema_and_missing_required_csv_fields() {
    let dir = tempfile::tempdir().unwrap();
    let json_path = dir.path().join("bad_result_log.json");
    std::fs::write(
        &json_path,
        r#"{"schema_version":"result-log.v2","games":[],"draft_picks":[]}"#,
    )
    .unwrap();
    let json_error = load_result_log(&json_path, Some("json")).unwrap_err();
    assert!(
        json_error
            .to_string()
            .contains("Unsupported result-log schema_version")
    );

    let csv_path = dir.path().join("bad_result_log.csv");
    std::fs::write(
        &csv_path,
        "record_type,match_id,game_number,queue,format,won,mulligans,sideboarded\n\
         game,m1,1,bo1,standard,true,0,false\n\
         ,m2,1,bo1,standard,false,1,false\n",
    )
    .unwrap();
    let csv_error = load_result_log(&csv_path, Some("csv")).unwrap_err();
    assert!(
        csv_error
            .to_string()
            .contains("missing required field record_type")
    );
}

#[test]
fn result_log_feeds_stats_and_report_summary() {
    let loaded = load_result_log("tests/fixtures/result_logs.json", Some("json")).unwrap();
    let constructed = summarize_constructed(&loaded.games);
    assert_eq!(constructed.games, 3);
    assert_eq!(constructed.matches, 2);
    assert_eq!(
        constructed
            .play_draw_performance
            .get("play")
            .unwrap()
            .successes,
        2
    );

    let summary = summarize_loaded_result_log(&loaded).unwrap();
    assert_eq!(summary["schema_version"], "result-log-summary.v1");
    assert_eq!(summary["source"]["game_count"], 3);
    assert_eq!(summary["source"]["draft_pick_count"], 2);
    assert!(summary["constructed"]["matchup_matrix"]["aggro"]["rate"].is_number());
    assert!(summary["draft_cards"]["Test Bear"]["card_win_rate"]["rate"].is_number());
}

#[test]
fn result_log_schemas_are_available() {
    let document_schema = schema_json("result-log").unwrap();
    let report_schema = schema_json("result-log-load-report").unwrap();
    assert!(document_schema.to_string().contains("ResultLogDocument"));
    assert!(report_schema.to_string().contains("ResultLogLoadReport"));
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
