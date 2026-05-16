use crate::analysis::{AnalysisInputs, build_analysis_report};
use crate::catalog::load_catalog_from_str;
use crate::export::arena::format_arena_decklist;
use crate::ingest::card_database::CardDatabase;
use crate::ingest::collection_parser::parse_collection_csv_from_str;
use crate::ingest::decklist_importer::parse_arena_decklist_text;
use crate::observability::source_snapshot::text_sha256;
use crate::report::{AnalysisReport, render_report};
use crate::result_log::load_result_log_from_str;
use crate::rules::validator::DeckValidator;
use crate::sim::opening_hand::simulate_opening_hands;
use axum::Router;
use axum::extract::Json;
use axum::extract::Path as AxumPath;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

const INDEX_HTML: &str = include_str!("web_assets/index.html");
const APP_CSS: &str = include_str!("web_assets/app.css");
const APP_JS: &str = include_str!("web_assets/app.js");

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

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub deck_text: String,
    pub catalog_text: String,
    #[serde(default = "default_auto")]
    pub catalog_format: String,
    pub collection_text: Option<String>,
    pub result_log_text: Option<String>,
    pub result_log_format: Option<String>,
    pub format_name: String,
    pub queue: Option<String>,
    pub trials: Option<u32>,
    pub seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ReportRenderRequest {
    pub report: AnalysisReport,
    pub output: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn local_only_routes() -> &'static [&'static str] {
    &[
        "GET /",
        "GET /assets/app.css",
        "GET /assets/app.js",
        "GET /api/health",
        "POST /api/analyze",
        "POST /api/report/render",
        "POST /validate",
        "POST /export",
        "POST /simulate-opening",
    ]
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/assets/{asset}", get(asset))
        .route("/api/health", get(api_health))
        .route("/api/analyze", post(analyze))
        .route("/api/report/render", post(render_report_api))
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

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn asset(AxumPath(asset): AxumPath<String>) -> impl IntoResponse {
    match asset.as_str() {
        "app.css" => ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], APP_CSS).into_response(),
        "app.js" => (
            [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
            APP_JS,
        )
            .into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn api_health() -> Json<Value> {
    Json(json!({
        "name": "mtg-deckbuilder",
        "mode": "local-only",
        "routes": local_only_routes(),
        "defaults": {
            "catalog_format": "auto",
            "format_name": "standard",
            "queue": "bo1",
            "trials": 1000,
            "seed": 1
        },
        "assumptions": [
            "Web adapter calls deterministic Rust services only.",
            "Uploaded text content is processed in memory and is not persisted by the server.",
            "No external card data, metagame, Arena client control, Steam login, scraping, tracking, overlay, or gameplay automation is performed.",
            "Reports are Bo1/Bo3-oriented and do not claim exact MTG Arena parity."
        ],
        "supports": {
            "catalog_formats": ["csv", "json", "jsonl", "yaml"],
            "result_log_formats": ["csv", "json", "jsonl"],
            "report_outputs": ["json", "markdown", "csv"]
        }
    }))
}

async fn analyze(
    Json(request): Json<AnalyzeRequest>,
) -> Result<Json<AnalysisReport>, (axum::http::StatusCode, Json<ErrorResponse>)> {
    let result = (|| -> anyhow::Result<AnalysisReport> {
        let catalog_format = format_or_infer(&request.catalog_format, &request.catalog_text);
        let catalog_label = format!("uploaded-catalog.{catalog_format}");
        let catalog = load_catalog_from_str(
            catalog_label,
            Some(catalog_format.as_str()),
            &request.catalog_text,
        )?;
        let deck = parse_arena_decklist_text(&request.deck_text)?;

        let collection = request
            .collection_text
            .as_deref()
            .filter(|text| !text.trim().is_empty())
            .map(|text| parse_collection_csv_from_str("uploaded-collection.csv", text, None, None))
            .transpose()?;
        let result_log = request
            .result_log_text
            .as_deref()
            .filter(|text| !text.trim().is_empty())
            .map(|text| {
                let format = request
                    .result_log_format
                    .as_deref()
                    .filter(|format| !format.trim().is_empty() && *format != "auto")
                    .map(str::to_ascii_lowercase)
                    .unwrap_or_else(|| infer_text_format(text).to_string());
                let label = format!("uploaded-result-log.{format}");
                load_result_log_from_str(label, Some(format.as_str()), text)
            })
            .transpose()?;

        let mut source_hashes = BTreeMap::new();
        source_hashes.insert("deck".to_string(), text_sha256(&request.deck_text));
        source_hashes.insert("cards".to_string(), text_sha256(&request.catalog_text));
        if let Some(collection_text) = request
            .collection_text
            .as_deref()
            .filter(|text| !text.trim().is_empty())
        {
            source_hashes.insert("collection".to_string(), text_sha256(collection_text));
        }
        if let Some(result_log_text) = request
            .result_log_text
            .as_deref()
            .filter(|text| !text.trim().is_empty())
        {
            source_hashes.insert("result_log".to_string(), text_sha256(result_log_text));
        }

        build_analysis_report(AnalysisInputs {
            deck,
            catalog,
            collection,
            result_log,
            format_name: request.format_name,
            queue: request.queue.unwrap_or_else(|| "bo1".to_string()),
            trials: request.trials.unwrap_or(1000),
            seed: request.seed.unwrap_or(1),
            source_hashes,
        })
    })();
    result.map(Json).map_err(error_response)
}

async fn render_report_api(
    Json(request): Json<ReportRenderRequest>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, Json<ErrorResponse>)> {
    let result = (|| -> anyhow::Result<(&'static str, String)> {
        if request.report.schema_version != "analysis-report.v1" {
            anyhow::bail!(
                "Unsupported report schema_version {:?}; expected \"analysis-report.v1\"",
                request.report.schema_version
            );
        }
        let output = request.output.to_ascii_lowercase();
        let content = render_report(&request.report, &output)?;
        Ok((content_type(&output), content))
    })();
    result
        .map(|(content_type, body)| ([(header::CONTENT_TYPE, content_type)], body))
        .map_err(error_response)
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

fn default_auto() -> String {
    "auto".to_string()
}

fn format_or_infer(format: &str, text: &str) -> String {
    let trimmed = format.trim();
    if trimmed.is_empty() || trimmed == "auto" {
        infer_text_format(text).to_string()
    } else {
        trimmed.to_ascii_lowercase()
    }
}

fn infer_text_format(text: &str) -> &'static str {
    let trimmed = text.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        "json"
    } else if trimmed
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.trim_start().starts_with('{'))
    {
        "jsonl"
    } else {
        "csv"
    }
}

fn content_type(output: &str) -> &'static str {
    match output {
        "json" => "application/json; charset=utf-8",
        "markdown" | "md" => "text/markdown; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        _ => "text/plain; charset=utf-8",
    }
}
