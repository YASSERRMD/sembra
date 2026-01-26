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
use rand::Rng;
use uuid::Uuid;

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

#[derive(Deserialize)]
pub struct IngestRequest {
    pub text: String,
    pub document_id: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct IngestResponse {
    pub chunk_count: usize,
    pub document_id: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub total_chunks: i64,
    pub components: ComponentStatus,
}

#[derive(Serialize)]
pub struct ComponentStatus {
    pub database: String,
    pub cache: String,
}

#[derive(Serialize)]
pub struct ConfigResponse {
    pub embedding_provider: String,
    pub vector_dim: usize,
}

/// Helper to generate mock embedding
fn generate_mock_embedding() -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..768).map(|_| rng.gen::<f32>()).collect()
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

/// System Status endpoint
async fn status_handler(State(state): State<Arc<RwLock<AppState>>>) -> Json<StatusResponse> {
    let state = state.read().await;
    let count = state.barq_db.chunk_count().await.unwrap_or(-1);
    let db_health = state.graph_db.health_check().await.unwrap_or(false);

    Json(StatusResponse {
        total_chunks: count,
        components: ComponentStatus {
            database: if db_health { "Connected".into() } else { "Disconnected".into() },
            cache: "In-Memory (Moka)".into(),
        },
    })
}

/// Config endpoint
async fn config_handler() -> Json<ConfigResponse> {
    Json(ConfigResponse {
        embedding_provider: "Mock (Random)".into(),
        vector_dim: 768,
    })
}

/// Ingest endpoint - Chunk and Store
async fn ingest_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, StatusCode> {
    let state = state.read().await;
    
    // Simple splitting by newline for now
    let chunks: Vec<&str> = request.text.split('\n').filter(|s| !s.trim().is_empty()).collect();
    let mut count = 0;

    for chunk_text in chunks {
        let chunk_id = format!("{}:{}", request.document_id, Uuid::new_v4());
        let embedding = generate_mock_embedding();
        
        // Store in BarqDB
        state.barq_db.insert_chunk(
            &chunk_id,
            &request.document_id,
            chunk_text,
            embedding,
            request.metadata.clone().unwrap_or(serde_json::json!({})),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert chunk: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        count += 1;
    }

    Ok(Json(IngestResponse {
        chunk_count: count,
        document_id: request.document_id,
    }))
}

/// Retrieve endpoint - search for relevant chunks
async fn retrieve_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<RetrieveRequest>,
) -> Result<Json<RetrieveResponse>, StatusCode> {
    let start = std::time::Instant::now();
    let top_k = request.top_k.unwrap_or(10);
    let state = state.read().await;

    // Generate mock embedding if missing (Simulation)
    let embedding = if request.query_embedding.is_empty() {
        generate_mock_embedding()
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
        .route("/v1/status", get(status_handler)) // New
        .route("/v1/config", get(config_handler)) // New
        .route("/v1/ingest", post(ingest_handler)) // New
        .route("/v1/retrieve", post(retrieve_handler))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive()) // Add CORS for ease
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
