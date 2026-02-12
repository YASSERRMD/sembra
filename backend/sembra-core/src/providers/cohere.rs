use super::llm::{LLMProvider, ChatMessage};
use super::EmbeddingProvider;
use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cohere LLM Provider
pub struct CohereLLMProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl CohereLLMProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            api_key,
            model,
            base_url: "https://api.cohere.ai/v1".to_string(),
        }
    }

    pub fn with_base_url(api_key: String, model: String, base_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            api_key,
            model,
            base_url,
        }
    }
}

#[derive(Serialize)]
struct CohereChatRequest {
    model: String,
    message: String,
    preamble: Option<String>,
    chat_history: Vec<CohereChatMessage>,
}

#[derive(Serialize)]
struct CohereChatMessage {
    role: String,
    message: String,
}

#[derive(Deserialize)]
struct CohereChatResponse {
    text: String,
}

#[async_trait]
impl LLMProvider for CohereLLMProvider {
    async fn complete(&self, messages: &[ChatMessage], _temperature: f32) -> Result<String> {
        // Extract system as preamble
        let preamble = messages.iter().find(|m| m.role == "system").map(|m| m.content.clone());
        
        // Build chat history (excluding system and last user message)
        let non_system: Vec<_> = messages.iter().filter(|m| m.role != "system").collect();
        let last_message = non_system.last().map(|m| m.content.clone()).unwrap_or_default();
        
        let chat_history: Vec<CohereChatMessage> = non_system
            .iter()
            .take(non_system.len().saturating_sub(1))
            .map(|m| CohereChatMessage {
                role: if m.role == "assistant" { "CHATBOT".to_string() } else { "USER".to_string() },
                message: m.content.clone(),
            })
            .collect();

        let req = CohereChatRequest {
            model: self.model.clone(),
            message: last_message,
            preamble,
            chat_history,
        };

        let res = self.client.post(format!("{}/chat", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await
            .context("Failed to send request to Cohere")?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Cohere API error: {}", error_text));
        }

        let body: CohereChatResponse = res.json().await.context("Failed to parse Cohere response")?;
        Ok(body.text)
    }

    fn name(&self) -> &str {
        "cohere"
    }
}

/// Cohere Embedding Provider
pub struct CohereEmbeddingProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl CohereEmbeddingProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
            api_key,
            model,
            base_url: "https://api.cohere.ai/v1".to_string(),
        }
    }
}

#[derive(Serialize)]
struct CohereEmbedRequest {
    model: String,
    texts: Vec<String>,
    input_type: String,
}

#[derive(Deserialize)]
struct CohereEmbedResponse {
    embeddings: Vec<Vec<f64>>,
}

#[async_trait]
impl EmbeddingProvider for CohereEmbeddingProvider {
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let req = CohereEmbedRequest {
            model: self.model.clone(),
            texts: vec![text.to_string()],
            input_type: "search_query".to_string(),
        };

        let res = self.client.post(format!("{}/embed", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await
            .context("Failed to send request to Cohere")?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Cohere Embed API error: {}", error_text));
        }

        let body: CohereEmbedResponse = res.json().await?;
        Ok(body.embeddings.first()
            .context("No embeddings returned")?
            .iter()
            .map(|&v| v as f32)
            .collect())
    }

    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let req = CohereEmbedRequest {
            model: self.model.clone(),
            texts: texts.to_vec(),
            input_type: "search_document".to_string(),
        };

        let res = self.client.post(format!("{}/embed", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await
            .context("Failed to send batch request to Cohere")?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Cohere Batch Embed error: {}", error_text));
        }

        let body: CohereEmbedResponse = res.json().await?;
        Ok(body.embeddings.iter()
            .map(|emb| emb.iter().map(|&v| v as f32).collect())
            .collect())
    }
}
