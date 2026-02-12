use super::llm::{LLMProvider, ChatMessage};
use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Anthropic Claude LLM Provider
pub struct AnthropicLLMProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl AnthropicLLMProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            api_key,
            model,
            base_url: "https://api.anthropic.com".to_string(),
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
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    system: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: String,
}

#[async_trait]
impl LLMProvider for AnthropicLLMProvider {
    async fn complete(&self, messages: &[ChatMessage], _temperature: f32) -> Result<String> {
        // Extract system message if present
        let system_msg = messages.iter().find(|m| m.role == "system").map(|m| m.content.clone());
        
        // Convert messages (excluding system)
        let anthropic_messages: Vec<AnthropicMessage> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| AnthropicMessage {
                role: if m.role == "assistant" { "assistant".to_string() } else { "user".to_string() },
                content: m.content.clone(),
            })
            .collect();

        let req = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: anthropic_messages,
            system: system_msg,
        };

        let res = self.client.post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&req)
            .send()
            .await
            .context("Failed to send request to Anthropic")?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }

        let body: AnthropicResponse = res.json().await.context("Failed to parse Anthropic response")?;
        Ok(body.content.first().map(|c| c.text.clone()).unwrap_or_default())
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}
