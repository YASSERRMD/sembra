use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use crate::state::AppState;
use sembra_core::providers::llm::{LLMProvider, ChatMessage, OpenAILLMProvider, OllamaLLMProvider};
use sembra_core::providers::anthropic::AnthropicLLMProvider;
use sembra_core::providers::groq::GroqLLMProvider;
use sembra_core::providers::cohere::CohereLLMProvider;

// --- Configuration ---
#[derive(Deserialize)]
pub struct ConfigureLLMRequest {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Serialize)]
pub struct ConfigureLLMResponse {
    pub status: String,
    pub provider: String,
    pub model: String,
}

pub async fn configure_llm(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<ConfigureLLMRequest>,
) ->  Result<Json<ConfigureLLMResponse>, (StatusCode, String)> {
    
    // Create new provider instance first (to validate)
    let new_provider: Arc<dyn LLMProvider> = match payload.provider.as_str() {
        "openai" => {
            let key = payload.api_key.clone().or(std::env::var("OPENAI_API_KEY").ok())
                .ok_or((StatusCode::BAD_REQUEST, "OpenAI API key required".into()))?;
            Arc::new(OpenAILLMProvider::new(key, payload.model.clone()))
        },
        "anthropic" => {
            let key = payload.api_key.clone().or(std::env::var("ANTHROPIC_API_KEY").ok())
                .ok_or((StatusCode::BAD_REQUEST, "Anthropic API key required".into()))?;
            if let Some(base_url) = payload.base_url.clone() {
                Arc::new(AnthropicLLMProvider::with_base_url(key, payload.model.clone(), base_url))
            } else {
                Arc::new(AnthropicLLMProvider::new(key, payload.model.clone()))
            }
        },
        "groq" => {
            let key = payload.api_key.clone().or(std::env::var("GROQ_API_KEY").ok())
                .ok_or((StatusCode::BAD_REQUEST, "Groq API key required".into()))?;
            if let Some(base_url) = payload.base_url.clone() {
                Arc::new(GroqLLMProvider::with_base_url(key, payload.model.clone(), base_url))
            } else {
                Arc::new(GroqLLMProvider::new(key, payload.model.clone()))
            }
        },
        "cohere" => {
            let key = payload.api_key.clone().or(std::env::var("COHERE_API_KEY").ok())
                .ok_or((StatusCode::BAD_REQUEST, "Cohere API key required".into()))?;
            if let Some(base_url) = payload.base_url.clone() {
                Arc::new(CohereLLMProvider::with_base_url(key, payload.model.clone(), base_url))
            } else {
                Arc::new(CohereLLMProvider::new(key, payload.model.clone()))
            }
        },
        "ollama" => {
            let url = payload.base_url.clone().unwrap_or_else(|| std::env::var("OLLAMA_BASE_URL").unwrap_or("http://localhost:11434".into()));
            Arc::new(OllamaLLMProvider::new(url, payload.model.clone()))
        },
        _ => return Err((StatusCode::BAD_REQUEST, format!("Unsupported provider: {}. Supported: openai, anthropic, groq, cohere, ollama", payload.provider))),
    };

    let mut state_write = state.write().await;
    
    // Save to Postgres
    state_write.metadata.set_config("llm_provider", &payload.provider).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state_write.metadata.set_config("llm_model", &payload.model).await
         .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Save API key with provider-specific key name
    if let Some(key) = &payload.api_key {
        let key_name = format!("{}_api_key", payload.provider);
        state_write.metadata.set_config(&key_name, key).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // Save base URL if provided
    if let Some(url) = &payload.base_url {
        let url_key = format!("{}_base_url", payload.provider);
        state_write.metadata.set_config(&url_key, url).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // Update state
    state_write.llm_provider = new_provider;

    Ok(Json(ConfigureLLMResponse {
        status: "success".into(),
        provider: payload.provider,
        model: payload.model,
    }))
}

// --- Ask / RAG ---
#[derive(Deserialize)]
pub struct AskRequest {
    pub query: String,
    pub include_graph: Option<bool>,
}

#[derive(Serialize)]
pub struct AskResponse {
    pub answer: String,
    pub context_snippets: Vec<String>,
}

pub async fn ask(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<AskRequest>,
) -> Result<Json<AskResponse>, (StatusCode, String)> {
    let state_read = state.read().await;

    // 1. Embed Query
    let embedding = state_read.embedding_provider.embed_query(&payload.query).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Embedding failed: {}", e)))?;

    // 2. Retrieve from BarqDB (Vector Search)
    let mut context_text = String::new();
    let mut snippets = Vec::new();
    
    let results = state_read.vector_db.search("sembra_chunks", embedding, 5).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vector search failed: {}", e)))?;

    tracing::info!("Search returned {} results for query: {}", results.len(), &payload.query);

    for res in results {
         if let Some(text) = res.payload.as_ref().and_then(|p| p.get("text")).and_then(|t| t.as_str()) {
             context_text.push_str(text);
             context_text.push_str("\n---\n");
             snippets.push(text.to_string());
         }
    }

    // 4. Call LLM
    let system_prompt = "You are SEMBRA, an enterprise AI assistant. Answer the user's question based strictly on the provided context. If the answer is not in the context, say so.";
    let user_prompt = format!("Context:\n{}\n\nQuestion: {}", context_text, payload.query);

    let messages = vec![
        ChatMessage { role: "system".into(), content: system_prompt.into() },
        ChatMessage { role: "user".into(), content: user_prompt },
    ];

    let answer = state_read.llm_provider.complete(&messages, 0.7).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("LLM generation failed: {}", e)))?;

    Ok(Json(AskResponse {
        answer,
        context_snippets: snippets,
    }))
}
