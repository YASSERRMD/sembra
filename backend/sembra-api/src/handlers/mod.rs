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
use uuid::Uuid;

use crate::state::AppState;
use crate::providers::{EmbeddingProvider, mock::MockProvider, openai::OpenAIProvider};
// Imports fixed below

// Fix imports
use sembra_types::{HealthResponse, RetrieveRequest, RetrieveResponse, RetrieveResult};
use sembra_graph::GraphDB;

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

#[derive(Deserialize)]
pub struct ConfigUpdateRequest {
    pub embedding_provider: Option<String>,
    pub openai_api_key: Option<String>,
}

/// Helper to get provider
async fn get_provider(state: &AppState) -> Box<dyn EmbeddingProvider> {
    let provider_name = state.barq_db.get_config("embedding_provider")
        .await.unwrap_or(None).unwrap_or("mock".to_string());
    
    if provider_name == "openai" {
        let api_key = state.barq_db.get_config("openai_api_key")
            .await.unwrap_or(None).unwrap_or("sk-mock".to_string());
        Box::new(OpenAIProvider::new(api_key, "text-embedding-3-small".to_string()))
    } else {
        Box::new(MockProvider::new())
    }
}

pub async fn health_handler(State(state): State<Arc<RwLock<AppState>>>) -> Json<HealthResponse> {
    let db_health = state.read().await.graph_db.health_check().await.unwrap_or(false);
    Json(HealthResponse {
        status: if db_health { "healthy".to_string() } else { "degraded".to_string() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: 0,
    })
}

pub async fn status_handler(State(state): State<Arc<RwLock<AppState>>>) -> Json<StatusResponse> {
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

pub async fn config_handler(State(state): State<Arc<RwLock<AppState>>>) -> Json<ConfigResponse> {
    let state = state.read().await;
    let provider = state.barq_db.get_config("embedding_provider")
        .await.unwrap_or(None).unwrap_or("mock".to_string());
    
    Json(ConfigResponse {
        embedding_provider: provider,
        vector_dim: 768,
    })
}

pub async fn update_config_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<ConfigUpdateRequest>,
) -> Result<Json<ConfigResponse>, StatusCode> {
    let state = state.read().await;
    
    if let Some(p) = req.embedding_provider {
        state.barq_db.set_config("embedding_provider", &p).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(k) = req.openai_api_key {
        state.barq_db.set_config("openai_api_key", &k).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Return updated
    let provider = state.barq_db.get_config("embedding_provider")
        .await.unwrap_or(None).unwrap_or("mock".to_string());

    Ok(Json(ConfigResponse {
        embedding_provider: provider,
        vector_dim: 768,
    }))
}

pub async fn ingest_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, StatusCode> {
    let state_read = state.read().await;
    let provider = get_provider(&state_read).await;
    
    let chunks: Vec<&str> = request.text.split('\n').filter(|s| !s.trim().is_empty()).collect();
    let mut count = 0;

    for chunk_text in chunks {
        let chunk_id = format!("{}:{}", request.document_id, Uuid::new_v4());
        let embedding = provider.embed_query(chunk_text).await.map_err(|e| {
            tracing::error!("Embedding failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        state_read.barq_db.insert_chunk(
            &chunk_id,
            &request.document_id,
            chunk_text,
            embedding,
            request.metadata.clone().unwrap_or(serde_json::json!({})),
        )
        .await
        .map_err(|e| {
            tracing::error!("Insert failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        count += 1;
    }

    Ok(Json(IngestResponse {
        chunk_count: count,
        document_id: request.document_id,
    }))
}

pub async fn retrieve_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<RetrieveRequest>,
) -> Result<Json<RetrieveResponse>, StatusCode> {
    let start = std::time::Instant::now();
    let top_k = request.top_k.unwrap_or(10);
    let state_read = state.read().await;

    let embedding = if request.query_embedding.is_empty() {
        let provider = get_provider(&state_read).await;
        provider.embed_query(&request.query).await.map_err(|e| {
            tracing::error!("Embedding failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    } else {
        request.query_embedding
    };

    let search_results = state_read.barq_db
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

// Login Structs
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let state = state.read().await;
    
    let user = state.barq_db.get_user_by_email(&req.email).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    let user = match user {
        Some(u) => u,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    if state.auth.verify_password(&user.password_hash, &req.password) {
        let token = state.auth.generate_token(&user.id, &user.role)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(LoginResponse { token }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn create_router(state: Arc<RwLock<AppState>>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/status", get(status_handler))
        .route("/v1/config", get(config_handler).post(update_config_handler))
        .route("/v1/ingest", post(ingest_handler))
        .route("/v1/retrieve", post(retrieve_handler))
        .route("/v1/login", post(login_handler)) // New route
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
}
