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
use sembra_storage::BarqDB;
use sembra_graph::GraphDB;
use sembra_cache::CelrixCache;
use sembra_types::{HealthResponse, RetrieveRequest, RetrieveResponse, RetrieveResult};

/// Application state shared across handlers
pub struct AppState {
    pub cache: CelrixCache,
    pub barq_db: BarqDB,
    pub graph_db: GraphDB,
}

/// Health check endpoint
async fn health_handler(State(state): State<Arc<RwLock<AppState>>>) -> Json<HealthResponse> {
    // Check DB health
    let db_health = state.read().await.graph_db.health_check().await.unwrap_or(false);
    
    Json(HealthResponse {
        status: if db_health { "healthy".to_string() } else { "degraded".to_string() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: 0,
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

    // 1. Check cache for query embedding (Simulated for this request scope, 
    //    normally would check cache for query string -> embedding mapping)
    
    // 2. Perform Hybrid Search via BarqDB
    // If request has no embedding, we would generate it here. 
    // For now we expect client/ingress to provide it or use mock.
    let embedding = if request.query_embedding.is_empty() {
        vec![0.0; 768] // Dimensionality placeholder
    } else {
        request.query_embedding
    };

    let search_results = state.barq_db
        .hybrid_search(embedding, &request.query, top_k, 60.0)
        .await
        .map_err(|e| {
            tracing::error!("Search failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let results: Vec<RetrieveResult> = search_results
        .into_iter()
        .map(|r| RetrieveResult {
            chunk_id: r.chunk_id,
            score: r.score,
            text: r.text,
            document_id: r.document_id,
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

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/sembra".to_string());
    
    // Initialize connections
    info!("Connecting to BarqDB (Storage & Graph) at {}", database_url);
    let barq_db = BarqDB::connect(&database_url, 10).await?;
    let graph_db = GraphDB::connect(&database_url, 10).await?;

    info!("Initializing Graph Schema...");
    if let Err(e) = graph_db.init_schema().await {
        tracing::warn!("Graph schema init warning (might already exist): {}", e);
    }

    // Create application state
    let state = Arc::new(RwLock::new(AppState {
        cache: CelrixCache::new(100_000, 86400),
        barq_db,
        graph_db,
    }));

    // Create router
    let app = create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
