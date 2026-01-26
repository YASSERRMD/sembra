use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMessage {
    pub chunk_id: String,
    pub text: String,
    pub metadata: Option<serde_json::Value>,
    pub embedding: Option<Vec<f32>>,
}
