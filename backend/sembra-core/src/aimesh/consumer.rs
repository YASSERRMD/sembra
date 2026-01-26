//! AiMesh Consumer - Client for AiMesh message queue
//!
//! Connects to AiMesh for high-performance message consumption

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// A chunk message received from AiMesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMessage {
    pub chunk_id: String,
    pub text: String,
    pub document_id: String,
    pub sequence_num: u64,
    pub metadata: serde_json::Value,
}

/// AiMesh message for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub agent_id: String,
    pub payload: Vec<u8>,
    pub token_budget: f64,
    pub deadline_unix_ms: i64,
}

/// AiMesh acknowledgement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessageAck {
    pub message_id: String,
    pub processing_latency_ms: u64,
    pub tokens_used: u64,
    pub routed_to: String,
}

/// AiMesh Consumer - connects to AiMesh message queue
pub struct AiMeshConsumer {
    client: reqwest::Client,
    base_url: String,
    topic: String,
    consumer_group: String,
}

impl AiMeshConsumer {
    /// Connect to AiMesh
    pub async fn connect(url: &str, topic: &str, consumer_group: &str) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: url.trim_end_matches('/').to_string(),
            topic: topic.to_string(),
            consumer_group: consumer_group.to_string(),
        })
    }

    /// Create from environment variable
    pub async fn from_env() -> Result<Self> {
        let base_url = std::env::var("AIMESH_URL")
            .unwrap_or_else(|_| "http://localhost:9000".to_string());
        Self::connect(&base_url, "sembra:chunks", "sembra_embeddings_v1").await
    }

    /// Read a batch of messages
    pub async fn read_batch(&self, batch_size: usize) -> Result<Vec<ChunkMessage>> {
        let url = format!(
            "{}/topics/{}/consume?group={}&batch_size={}",
            self.base_url, self.topic, self.consumer_group, batch_size
        );

        let resp = self.client.get(&url).send().await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to consume messages: {} - {}", status, text);
        }

        let messages: Vec<ChunkMessage> = resp.json().await?;
        Ok(messages)
    }

    /// Commit a processed batch
    pub async fn commit_batch(&self, batch_id: &str) -> Result<()> {
        let url = format!(
            "{}/topics/{}/commit?group={}&batch_id={}",
            self.base_url, self.topic, self.consumer_group, batch_id
        );

        let resp = self.client.post(&url).send().await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to commit batch: {} - {}", status, text);
        }

        Ok(())
    }

    /// Health check
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.status().is_success())
    }

    /// Get topic name
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Get consumer group
    pub fn consumer_group(&self) -> &str {
        &self.consumer_group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_message_serialization() {
        let msg = ChunkMessage {
            chunk_id: "chunk:1".to_string(),
            text: "Hello world".to_string(),
            document_id: "doc:1".to_string(),
            sequence_num: 1,
            metadata: serde_json::json!({}),
        };

        let json = serde_json::to_string(&msg).expect("Serialize");
        let parsed: ChunkMessage = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(parsed.chunk_id, "chunk:1");
    }

    #[test]
    fn test_ai_message_serialization() {
        let msg = AiMessage {
            agent_id: "agent:1".to_string(),
            payload: b"test".to_vec(),
            token_budget: 1000.0,
            deadline_unix_ms: 0,
        };

        let json = serde_json::to_string(&msg).expect("Serialize");
        let parsed: AiMessage = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(parsed.agent_id, "agent:1");
    }
}
