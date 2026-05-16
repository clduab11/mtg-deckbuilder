use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use mtgdeckbuilder::catalog::load_catalog_from_str;
use mtgdeckbuilder::ingest::collection_parser::parse_collection_csv_from_str;
use mtgdeckbuilder::result_log::load_result_log_from_str;
use serde_json::{Value, json};
use tower::ServiceExt;

const DECK_TEXT: &str = include_str!("../examples/sample_deck.txt");
const CATALOG_JSON: &str = include_str!("fixtures/cards_scryfall.json");
const COLLECTION_CSV: &str = include_str!("fixtures/collection.csv");
const RESULT_LOG_JSON: &str = include_str!("fixtures/result_logs.json");

async fn get_json(path: &str) -> (StatusCode, Value) {
    let response = mtgdeckbuilder::web::router()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn post_json(path: &str, payload: Value) -> (StatusCode, Value) {
    let response = mtgdeckbuilder::web::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn post_text(path: &str, payload: Value) -> (StatusCode, String) {
    let response = mtgdeckbuilder::web::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[test]
fn in_memory_loaders_accept_uploaded_text_without_paths() {
    let catalog = load_catalog_from_str("fixture.json", Some("json"), CATALOG_JSON).unwrap();
    assert_eq!(catalog.report.card_count, 15);
    assert!(catalog.database.get("Test Bear").is_some());

    let collection =
        parse_collection_csv_from_str("fixture-collection.csv", COLLECTION_CSV, None, None)
            .unwrap();
    assert_eq!(collection.owned("Test Bear"), 4);
    assert_eq!(
        collection.source_path.as_deref(),
        Some("fixture-collection.csv")
    );

    let result_log =
        load_result_log_from_str("fixture-result-log.json", Some("json"), RESULT_LOG_JSON).unwrap();
    assert_eq!(result_log.report.game_count, 3);
    assert_eq!(result_log.report.draft_pick_count, 2);
}

#[tokio::test]
async fn health_reports_local_api_contracts() {
    let (status, body) = get_json("/api/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mode"], "local-only");
    assert!(
        body["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route == "POST /api/analyze")
    );
}

#[tokio::test]
async fn analyze_and_render_report_use_in_memory_uploads() {
    let (status, body) = post_json(
        "/api/analyze",
        json!({
            "deck_text": DECK_TEXT,
            "catalog_text": CATALOG_JSON,
            "catalog_format": "json",
            "collection_text": COLLECTION_CSV,
            "result_log_text": RESULT_LOG_JSON,
            "result_log_format": "json",
            "format_name": "standard",
            "queue": "bo1",
            "trials": 25,
            "seed": 7
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["schema_version"], "analysis-report.v1");
    assert_eq!(body["validation"]["status"], "PASS");
    assert_eq!(body["result_log"]["source"]["game_count"], 3);
    assert!(body["source_hashes"]["cards"].as_str().is_some());

    let (render_status, render_body) = post_text(
        "/api/report/render",
        json!({
            "report": body.clone(),
            "output": "markdown"
        }),
    )
    .await;
    assert_eq!(render_status, StatusCode::OK, "{render_body}");
    assert!(render_body.contains("MTG Deck Analysis Report"));
}

#[tokio::test]
async fn analyze_rejects_invalid_uploaded_content() {
    let (status, body) = post_json(
        "/api/analyze",
        json!({
            "deck_text": "this is not a decklist",
            "catalog_text": CATALOG_JSON,
            "catalog_format": "json",
            "format_name": "standard",
            "queue": "bo1",
            "trials": 10,
            "seed": 1
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("expected '<count> <card name>'")
    );
}
