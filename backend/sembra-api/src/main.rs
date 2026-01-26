//! SEMBRA API - REST API for document retrieval

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

// Re-export crates for use in handlers
use sembra_cache::CelrixCache;

/// Application state shared across handlers
pub struct AppState {
    pub cache: CelrixCache,
    // Add database connections when ready
}

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

/// Retrieve request body
#[derive(Deserialize)]
pub struct RetrieveRequest {
    pub query: String,
    pub top_k: Option<usize>,
    #[serde(default)]
    pub query_embedding: Vec<f32>,
}

/// Retrieve response
#[derive(Serialize)]
pub struct RetrieveResponse {
    pub results: Vec<RetrieveResult>,
    pub latency_ms: u64,
}

#[derive(Serialize)]
pub struct RetrieveResult {
    pub chunk_id: String,
    pub score: f32,
    pub text: Option<String>,
}

/// Health check endpoint
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: 0, // TODO: track actual uptime
    })
}

/// Retrieve endpoint - search for relevant chunks
async fn retrieve_handler(
    State(_state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<RetrieveRequest>,
) -> Result<Json<RetrieveResponse>, StatusCode> {
    let start = std::time::Instant::now();
    let top_k = request.top_k.unwrap_or(10);

    // For now, return mock results
    // In production, this would query BarqDB
    let results: Vec<RetrieveResult> = (0..top_k.min(5))
        .map(|i| RetrieveResult {
            chunk_id: format!("chunk:{}", i),
            score: 0.95 - (i as f32 * 0.1),
            text: Some(format!("Sample result for query: {}", request.query)),
        })
        .collect();

    let latency_ms = start.elapsed().as_millis() as u64;

    Ok(Json(RetrieveResponse { results, latency_ms }))
}

/// Create the API router
pub fn create_router(state: Arc<RwLock<AppState>>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/retrieve", post(retrieve_handler))
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("sembra_api=info".parse().unwrap()),
        )
        .init();

    info!("Starting SEMBRA API server...");

    // Create application state
    let state = Arc::new(RwLock::new(AppState {
        cache: CelrixCache::new(100_000, 86400),
    }));

    // Create router
    let app = create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
