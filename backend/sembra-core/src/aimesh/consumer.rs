//! AiMesh Consumer - consumes chunk messages from message broker

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// A chunk message received from AiMesh broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMessage {
    pub chunk_id: String,
    pub text: String,
    pub document_id: String,
    pub sequence_num: u64,
    pub metadata: serde_json::Value,
}

/// Consumer for reading chunk messages from AiMesh broker
pub struct AiMeshConsumer {
    pub broker_url: String,
    pub topic: String,
    pub consumer_group: String,
}

impl AiMeshConsumer {
    /// Connect to the AiMesh broker
    pub async fn connect_to_broker(url: &str) -> Result<Self> {
        Ok(Self {
            broker_url: url.to_string(),
            topic: "sembra:chunks".to_string(),
            consumer_group: "sembra_embeddings_v1".to_string(),
        })
    }

    /// Read a batch of chunk messages
    pub async fn read_batch(&self, batch_size: usize) -> Result<Vec<ChunkMessage>> {
        // For now, return mock data for testing
        Ok((0..batch_size)
            .map(|i| ChunkMessage {
                chunk_id: format!("chunk:{}", i),
                text: format!("Sample chunk text {}", i),
                document_id: "doc:test".to_string(),
                sequence_num: i as u64,
                metadata: serde_json::json!({}),
            })
            .collect())
    }

    /// Commit a processed batch
    pub async fn commit_batch(&self, _batch_id: String) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_to_broker() {
        let consumer = AiMeshConsumer::connect_to_broker("localhost:9092")
            .await
            .expect("Failed to connect");
        assert_eq!(consumer.topic, "sembra:chunks");
    }

    #[tokio::test]
    async fn test_read_batch() {
        let consumer = AiMeshConsumer::connect_to_broker("localhost:9092")
            .await
            .unwrap();
        let batch = consumer.read_batch(10).await.unwrap();
        assert_eq!(batch.len(), 10);
        assert_eq!(batch[0].chunk_id, "chunk:0");
    }
}
