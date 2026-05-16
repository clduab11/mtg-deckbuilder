use crate::export::arena::format_arena_decklist;
use crate::ingest::card_database::CardDatabase;
use crate::ingest::decklist_importer::parse_arena_decklist_text;
use crate::rules::validator::DeckValidator;
use crate::sim::opening_hand::simulate_opening_hands;
use axum::Router;
use axum::extract::Json;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub deck_text: String,
    pub card_data_path: String,
    pub format_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub deck_text: String,
}

#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    pub deck_text: String,
    pub card_data_path: String,
    pub trials: Option<u32>,
    pub seed: Option<u64>,
    pub mode: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn local_only_routes() -> &'static [&'static str] {
    &[
        "GET /",
        "POST /validate",
        "POST /export",
        "POST /simulate-opening",
    ]
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/validate", post(validate))
        .route("/export", post(export))
        .route("/simulate-opening", post(simulate_opening))
        .layer(CorsLayer::permissive())
}

pub async fn run(addr: SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router()).await?;
    Ok(())
}

async fn index() -> Json<Value> {
    Json(json!({
        "name": "mtg-deckbuilder",
        "mode": "local-only",
        "routes": local_only_routes(),
        "assumptions": [
            "Web adapter calls deterministic Rust services only.",
            "No external card data, metagame, or Arena client control is performed."
        ]
    }))
}

async fn validate(
    Json(request): Json<ValidateRequest>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<ErrorResponse>)> {
    let result = (|| -> anyhow::Result<Value> {
        let db = CardDatabase::from_scryfall_file(&request.card_data_path)?;
        let deck = parse_arena_decklist_text(&request.deck_text)?;
        Ok(serde_json::to_value(DeckValidator::new(db).validate(
            &deck,
            &request.format_name,
            None,
            false,
            None,
        ))?)
    })();
    result.map(Json).map_err(error_response)
}

async fn export(
    Json(request): Json<ExportRequest>,
) -> Result<String, (axum::http::StatusCode, Json<ErrorResponse>)> {
    let result = (|| -> anyhow::Result<String> {
        let deck = parse_arena_decklist_text(&request.deck_text)?;
        Ok(format_arena_decklist(&deck))
    })();
    result.map_err(error_response)
}

async fn simulate_opening(
    Json(request): Json<SimulateRequest>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<ErrorResponse>)> {
    let result = (|| -> anyhow::Result<Value> {
        let db = CardDatabase::from_scryfall_file(&request.card_data_path)?;
        let deck = parse_arena_decklist_text(&request.deck_text)?;
        Ok(serde_json::to_value(simulate_opening_hands(
            &deck,
            &db,
            request.mode.as_deref().unwrap_or("arena_n2"),
            request.trials.unwrap_or(1000),
            request.seed.unwrap_or(1),
            2,
        )?)?)
    })();
    result.map(Json).map_err(error_response)
}

fn error_response(error: anyhow::Error) -> (axum::http::StatusCode, Json<ErrorResponse>) {
    (
        axum::http::StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: error.to_string(),
        }),
    )
}
