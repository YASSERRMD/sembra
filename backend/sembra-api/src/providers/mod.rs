use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

pub mod openai;
pub mod mock;
