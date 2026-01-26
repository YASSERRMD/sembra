//! SEMBRA API - REST API for document retrieval
//!
//! Connects to Barq-DB, Barq-GraphDB, and uses Celrix Cache

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

use sembra_cache::CelrixCache;
use sembra_storage::BarqDBClient;
use sembra_graph::BarqGraphDBClient;

/// Application state shared across handlers
pub struct AppState {
    pub cache: CelrixCache,
    pub barq_db: BarqDBClient,
    pub barq_graphdb: BarqGraphDBClient,
}

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub barq_db: String,
    pub barq_graphdb: String,
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
    pub id: u64,
    pub score: f32,
    pub payload: Option<serde_json::Value>,
}

/// Health check endpoint
async fn health_handler(State(state): State<Arc<RwLock<AppState>>>) -> Json<HealthResponse> {
    let state = state.read().await;
    
    let barq_db_status = match state.barq_db.health().await {
        Ok(true) => "healthy",
        _ => "unhealthy",
    };
    
    let barq_graphdb_status = match state.barq_graphdb.health().await {
        Ok(true) => "healthy",
        _ => "unhealthy",
    };

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        barq_db: barq_db_status.to_string(),
        barq_graphdb: barq_graphdb_status.to_string(),
    })
}

/// Retrieve endpoint - search for relevant chunks
async fn retrieve_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<RetrieveRequest>,
) -> Result<Json<RetrieveResponse>, StatusCode> {
    let start = std::time::Instant::now();
    let top_k = request.top_k.unwrap_or(10);
    let state = state.read().await;

    // Check cache first
    let cache_key = format!("search:{}", request.query);
    if let Some(cached) = state.cache.get(&cache_key).await {
        info!("Cache hit for query: {}", request.query);
        // Return cached results (simplified for now)
        return Ok(Json(RetrieveResponse {
            results: vec![],
            latency_ms: start.elapsed().as_millis() as u64,
        }));
    }

    // Search Barq-DB
    let results = if !request.query_embedding.is_empty() {
        match state.barq_db.hybrid_search(
            "sembra_chunks",
            request.query_embedding.clone(),
            &request.query,
            top_k,
        ).await {
            Ok(results) => results,
            Err(e) => {
                tracing::error!("Search failed: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        // Fallback to vector search only if no embedding provided
        match state.barq_db.search(
            "sembra_chunks",
            vec![0.0; 384], // placeholder
            top_k,
        ).await {
            Ok(results) => results,
            Err(e) => {
                tracing::error!("Search failed: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    let response_results: Vec<RetrieveResult> = results
        .into_iter()
        .map(|r| RetrieveResult {
            id: r.id,
            score: r.score,
            payload: r.payload,
        })
        .collect();

    let latency_ms = start.elapsed().as_millis() as u64;

    Ok(Json(RetrieveResponse {
        results: response_results,
        latency_ms,
    }))
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

    // Initialize clients
    let barq_db = BarqDBClient::from_env()?;
    let barq_graphdb = BarqGraphDBClient::from_env()?;

    info!("Connected to Barq-DB and Barq-GraphDB");

    // Create application state
    let state = Arc::new(RwLock::new(AppState {
        cache: CelrixCache::new(100_000, 86400),
        barq_db,
        barq_graphdb,
    }));

    // Create router
    let app = create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
