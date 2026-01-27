use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::{Result, Context};
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, messages: &[ChatMessage], temperature: f32) -> Result<String>;
    fn name(&self) -> &str;
}

// --- OpenAI Implementation ---
pub struct OpenAILLMProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAILLMProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
            api_key,
            model,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
}

#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: ChatMessage,
}

#[async_trait]
impl LLMProvider for OpenAILLMProvider {
    async fn complete(&self, messages: &[ChatMessage], temperature: f32) -> Result<String> {
        let req = OpenAIChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            temperature,
        };

        let res = self.client.post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let body: OpenAIChatResponse = res.json().await?;
        Ok(body.choices.first().context("No choices returned")?.message.content.clone())
    }

    fn name(&self) -> &str {
        "openai"
    }
}

// --- Ollama Implementation ---
pub struct OllamaLLMProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaLLMProvider {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            base_url,
            model,
        }
    }
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: ChatMessage,
}

#[async_trait]
impl LLMProvider for OllamaLLMProvider {
    async fn complete(&self, messages: &[ChatMessage], temperature: f32) -> Result<String> {
        let req = OllamaChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            stream: false,
            options: OllamaOptions { temperature },
        };

        let res = self.client.post(format!("{}/api/chat", self.base_url))
            .json(&req)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Ollama API error: {}", error_text));
        }

        let body: OllamaChatResponse = res.json().await?;
        Ok(body.message.content)
    }

    fn name(&self) -> &str {
        "ollama"
    }
}
