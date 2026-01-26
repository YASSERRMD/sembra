use super::EmbeddingProvider;
use anyhow::Result;
use async_trait::async_trait;
use rand::Rng;

pub struct MockProvider;

impl MockProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EmbeddingProvider for MockProvider {
    async fn embed_query(&self, _text: &str) -> Result<Vec<f32>> {
        let mut rng = rand::thread_rng();
        Ok((0..768).map(|_| rng.gen::<f32>()).collect())
    }

    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut rng = rand::thread_rng();
        Ok(texts.iter().map(|_| (0..768).map(|_| rng.gen::<f32>()).collect()).collect())
    }
}
