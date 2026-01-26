use super::EmbeddingProvider;
use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIProvider {
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let res = self.client.post("https://api.openai.com/v1/embeddings")
            .bearer_auth(&self.api_key)
            .json(&json!({
                "input": text,
                "model": self.model
            }))
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !res.status().is_success() {
             let error_text = res.text().await?;
             return Err(anyhow::anyhow!("OpenAI API Error: {}", error_text));
        }

        let body: serde_json::Value = res.json().await.context("Failed to parse OpenAI response")?;
        let embedding = body["data"][0]["embedding"]
            .as_array()
            .context("Invalid response format: missing data[0].embedding")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();
        
        Ok(embedding)
    }

    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Batch embedding implementation
         let res = self.client.post("https://api.openai.com/v1/embeddings")
            .bearer_auth(&self.api_key)
            .json(&json!({
                "input": texts,
                "model": self.model
            }))
            .send()
            .await
            .context("Failed to send batch request")?;

        if !res.status().is_success() {
             let error_text = res.text().await?;
             return Err(anyhow::anyhow!("OpenAI Batch Error: {}", error_text));
        }

        let body: serde_json::Value = res.json().await.context("Failed to parse response")?;
        let data = body["data"].as_array().context("Missing data array")?;
        
        let mut embeddings = Vec::new();
        for item in data {
            let vec: Vec<f32> = item["embedding"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();
            embeddings.push(vec);
        }
        
        Ok(embeddings)
    }
}
