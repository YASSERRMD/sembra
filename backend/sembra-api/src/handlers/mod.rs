use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::state::AppState;

// ==================== Response Types ====================

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub postgres: String,
    pub barq_db: String,
    pub cache: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub total_chunks: u64,
    pub components: ComponentStatus,
}

#[derive(Serialize)]
pub struct ComponentStatus {
    pub postgres: String,
    pub barq_db: String,
    pub graph_db: String,
    pub cache: String,
    pub aimesh: String,
}

#[derive(Deserialize)]
pub struct IngestRequest {
    pub text: String,
    pub document_id: String,
}

#[derive(Serialize)]
pub struct IngestResponse {
    pub chunk_count: usize,
    pub document_id: String,
}

#[derive(Deserialize)]
pub struct RetrieveRequest {
    pub query: String,
    pub top_k: Option<usize>,
}

#[derive(Serialize)]
pub struct RetrieveResponse {
    pub results: Vec<RetrieveResult>,
    pub latency_ms: u64,
}

#[derive(Serialize, Deserialize)]
pub struct RetrieveResult {
    pub chunk_id: u64,
    pub document_id: String,
    pub text: String,
    pub score: f32,
}

#[derive(Deserialize)]
pub struct ConfigRequest {
    pub embedding_provider: Option<String>,
    pub openai_api_key: Option<String>,
}

#[derive(Serialize)]
pub struct ConfigResponse {
    pub embedding_provider: String,
    pub vector_dim: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

// ==================== Helpers ====================

fn hash_string_to_u64(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn split_text(text: &str, chunk_size: usize) -> Vec<String> {
    text.chars()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|c| c.iter().collect())
        .collect()
}

// ==================== Handlers ====================

pub async fn health_handler(State(state): State<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let state = state.read().await;
    let pg_ok = state.metadata.health().await.unwrap_or(false);
    let barq_ok = state.vector_db.health().await.unwrap_or(false);
    
    Json(HealthResponse {
        status: if pg_ok && barq_ok { "healthy".into() } else { "degraded".into() },
        postgres: if pg_ok { "connected".into() } else { "error".into() },
        barq_db: if barq_ok { "connected".into() } else { "error".into() },
        cache: "connected".into(),
    })
}

pub async fn status_handler(State(state): State<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let state = state.read().await;
    let pg_ok = state.metadata.health().await.unwrap_or(false);
    let barq_ok = state.vector_db.health().await.unwrap_or(false);
    let graph_ok = state.graph_db.health().await.unwrap_or(false);
    
    // Check Celrix (Cache)
    let cache_ok = state.cache.health().await.unwrap_or(false);

    // Check AiMesh connection
    let aimesh_ok = sembra_core::aimesh::AiMeshConsumer::from_env().await.is_ok();
    
    // Get actual chunk count from BarqDB
    let total_chunks = state.vector_db.search("sembra_chunks", vec![0.0f32; 384], 10000).await
        .map(|r| r.len() as u64)
        .unwrap_or(0);
    
    Json(StatusResponse {
        total_chunks,
        components: ComponentStatus {
            postgres: if pg_ok { "Connected".into() } else { "Error".into() },
            barq_db: if barq_ok { "Connected".into() } else { "Error".into() },
            graph_db: if graph_ok { "Connected".into() } else { "Error".into() },
            cache: if cache_ok { "Connected".into() } else { "Error".into() },
            aimesh: if aimesh_ok { "Connected".into() } else { "Error".into() },
        }
    })
}

#[derive(Deserialize)]
pub struct EmbeddingConfigRequest {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub batch_size: Option<usize>,
}

#[derive(Serialize)]
pub struct EmbeddingConfigResponse {
    pub status: String,
    pub provider: String,
    pub model: String,
}

#[derive(Serialize)]
pub struct FullConfigResponse {
    pub embedding: EmbeddingConfigResponse,
    pub llm: LLMConfigInfo,
    pub documents: u64,
}

#[derive(Serialize)]
pub struct LLMConfigInfo {
    pub provider: String,
    pub model: String,
}

#[derive(Serialize)]
pub struct DocumentInfo {
    pub document_id: String,
    pub name: String,
    pub chunk_count: u64,
}

pub async fn config_handler(State(state): State<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let state = state.read().await;
    let emb_provider = state.metadata.get_config("embedding_provider").await
        .ok().flatten().unwrap_or_else(|| "not configured".into());
    let emb_model = state.metadata.get_config("embedding_model").await
        .ok().flatten().unwrap_or_else(|| "not configured".into());
    let llm_provider = state.metadata.get_config("llm_provider").await
        .ok().flatten().unwrap_or_else(|| "not configured".into());
    let llm_model = state.metadata.get_config("llm_model").await
        .ok().flatten().unwrap_or_else(|| "not configured".into());
    
    let total_chunks = state.vector_db.search("sembra_chunks", vec![0.0f32; 384], 10000).await
        .map(|r| r.len() as u64)
        .unwrap_or(0);
    
    Json(FullConfigResponse {
        embedding: EmbeddingConfigResponse {
            status: "active".into(),
            provider: emb_provider,
            model: emb_model,
        },
        llm: LLMConfigInfo {
            provider: llm_provider,
            model: llm_model,
        },
        documents: total_chunks,
    })
}

pub async fn documents_handler(State(state): State<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let state = state.read().await;
    
    // Query graph for Document nodes
    let docs = state.graph_db.get_nodes_by_label("Document").await
        .unwrap_or_default();
    
    let doc_list: Vec<DocumentInfo> = docs.iter().filter_map(|node| {
        let props = &node.properties;
        Some(DocumentInfo {
            document_id: props.get("document_id")?.as_str()?.to_string(),
            name: props.get("name")?.as_str()?.to_string(),
            chunk_count: props.get("chunk_count").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    }).collect();
    
    Json(doc_list)
}

pub async fn configure_embedding_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<EmbeddingConfigRequest>,
) -> impl IntoResponse {
    let state = state.read().await;
    
    // Save configuration
    state.metadata.set_config("embedding_provider", &req.provider).await.ok();
    state.metadata.set_config("embedding_model", &req.model).await.ok();
    
    // Save provider-specific API key
    if let Some(key) = &req.api_key {
        let key_name = format!("{}_embedding_api_key", req.provider);
        state.metadata.set_config(&key_name, key).await.ok();
    }
    
    // Save base URL if provided
    if let Some(url) = &req.base_url {
        let url_key = format!("{}_embedding_base_url", req.provider);
        state.metadata.set_config(&url_key, url).await.ok();
    }
    
    if let Some(batch) = req.batch_size {
        state.metadata.set_config("embedding_batch_size", &batch.to_string()).await.ok();
    }
    
    // TODO: Re-initialize the embedding provider dynamically
    // For now, we just save the config
    
    Json(EmbeddingConfigResponse {
        status: "configured".into(),
        provider: req.provider,
        model: req.model,
    })
}

pub async fn ingest_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, StatusCode> {
    let state = state.read().await;
    let chunks = split_text(&req.text, 512);
    let count = chunks.len();
    
    // Ensure collection exists in BarqDB
    state.vector_db.create_collection("sembra_chunks", 384, "cosine").await.ok();
    
    for (i, chunk_text) in chunks.iter().enumerate() {
        let chunk_id = hash_string_to_u64(&format!("{}_{}", req.document_id, i));
        let vector = vec![0.1; 384]; // Placeholder embedding
        
        let payload = serde_json::json!({
            "document_id": req.document_id,
            "text": chunk_text
        });
        
        state.vector_db.insert("sembra_chunks", chunk_id, vector, payload).await
            .map_err(|e| {
                tracing::error!("Insert failed: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    Ok(Json(IngestResponse {
        chunk_count: count,
        document_id: req.document_id,
    }))
}

pub async fn retrieve_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<RetrieveRequest>,
) -> Result<Json<RetrieveResponse>, StatusCode> {
    let state = state.read().await;
    let start = std::time::Instant::now();
    
    let vector = vec![0.1; 384]; // Placeholder query vector
    
    let results = state.vector_db.hybrid_search("sembra_chunks", vector, &req.query, req.top_k.unwrap_or(5)).await
        .map_err(|e| {
             tracing::error!("Search failed: {}", e);
             StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
    let results = results.into_iter().map(|r| {
        let text = r.payload.as_ref()
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("").to_string();
        let doc_id = r.payload.as_ref()
            .and_then(|p| p.get("document_id"))
            .and_then(|t| t.as_str())
            .unwrap_or("unknown").to_string();
            
        RetrieveResult {
             chunk_id: r.id,
             document_id: doc_id,
             text,
             score: r.score
        }
    }).collect();

    Ok(Json(RetrieveResponse {
        results,
        latency_ms: start.elapsed().as_millis() as u64,
    }))
}

pub async fn login_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let state = state.read().await;
    
    // Lookup user in Postgres
    let user = state.metadata.get_user_by_email(&req.email).await
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

pub mod upload;
pub mod llm;

// ==================== Router ====================

pub fn create_router(state: Arc<RwLock<AppState>>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/status", get(status_handler))
        .route("/v1/config", get(config_handler))
        .route("/v1/documents", get(documents_handler))
        .route("/v1/configure-embedding", post(configure_embedding_handler))
        .route("/v1/ingest", post(ingest_handler))
        .route("/v1/retrieve", post(retrieve_handler))
        .route("/v1/login", post(login_handler))
        .route("/v1/upload", post(upload::upload_handler))
        .route("/v1/configure-llm", post(llm::configure_llm))
        .route("/v1/ask", post(llm::ask))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
}
