use super::llm::{LLMProvider, ChatMessage};
use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Groq LLM Provider (OpenAI-compatible API)
pub struct GroqLLMProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl GroqLLMProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
            api_key,
            model,
            base_url: "https://api.groq.com/openai/v1".to_string(),
        }
    }

    pub fn with_base_url(api_key: String, model: String, base_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
            api_key,
            model,
            base_url,
        }
    }
}

#[derive(Serialize)]
struct GroqChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize)]
struct GroqChatResponse {
    choices: Vec<GroqChoice>,
}

#[derive(Deserialize)]
struct GroqChoice {
    message: ChatMessage,
}

#[async_trait]
impl LLMProvider for GroqLLMProvider {
    async fn complete(&self, messages: &[ChatMessage], temperature: f32) -> Result<String> {
        let req = GroqChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            temperature,
        };

        let res = self.client.post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await
            .context("Failed to send request to Groq")?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Groq API error: {}", error_text));
        }

        let body: GroqChatResponse = res.json().await.context("Failed to parse Groq response")?;
        Ok(body.choices.first().context("No choices returned")?.message.content.clone())
    }

    fn name(&self) -> &str {
        "groq"
    }
}
