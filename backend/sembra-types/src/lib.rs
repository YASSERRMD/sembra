use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMessage {
    pub chunk_id: String,
    pub text: String,
    pub metadata: Option<serde_json::Value>,
    pub embedding: Option<Vec<f32>>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

/// Retrieve request body
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetrieveRequest {
    pub query: String,
    pub top_k: Option<i32>,
    #[serde(default)]
    pub query_embedding: Vec<f32>,
}

/// Retrieve response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveResponse {
    pub results: Vec<RetrieveResult>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveResult {
    pub chunk_id: String,
    pub score: f32,
    pub text: String,
    pub document_id: String,
}
