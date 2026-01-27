use super::EmbeddingProvider;
use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaProvider {
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);
        
        let res = self.client.post(&url)
            .json(&json!({
                "model": self.model,
                "prompt": text
            }))
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !res.status().is_success() {
             let error_text = res.text().await?;
             return Err(anyhow::anyhow!("Ollama API Error: {}", error_text));
        }

        let body: serde_json::Value = res.json().await.context("Failed to parse Ollama response")?;
        
        let embedding = body["embedding"]
            .as_array()
            .context("Invalid response format: missing embedding field")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();
        
        Ok(embedding)
    }

    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Ollama doesn't support batch embeddings natively in the /api/embeddings endpoint consistently across versions
        // We'll implement sequential processing for now
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed_query(text).await?);
        }
        Ok(embeddings)
    }
}
