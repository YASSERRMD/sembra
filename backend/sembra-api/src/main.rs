use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use sembra_storage::{MetadataStore, BarqDBClient};
use sembra_graph::BarqGraphDBClient;
use sembra_cache::CelrixCache;

mod config;
mod handlers;
mod state;
mod services;
mod extractors;
mod chunking;

use state::AppState;
use services::auth::AuthService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("sembra_api=info".parse().unwrap()),
        )
        .init();

    info!("Starting SEMBRA API server (Enterprise Edition)...");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/sembra".to_string());

    // Initialize Postgres MetadataStore
    info!("Connecting to Postgres...");
    let metadata = MetadataStore::connect(&database_url, 10).await?;
    
    info!("Initializing Postgres schema...");
    metadata.init_schema().await?;

    // Initialize BarqDB Client
    info!("Connecting to BarqDB...");
    let vector_db = BarqDBClient::from_env()?;

    // Initialize BarqGraphDB Client
    info!("Connecting to BarqGraphDB...");
    let graph_db = BarqGraphDBClient::from_env()?;

    // Initialize Celrix Cache
    info!("Connecting to Celrix Cache...");
    let cache = CelrixCache::from_env();

    // Initialize Auth Service
    let auth_service = AuthService::new(
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "enterprise_secret_key_change_me".to_string())
    );

    // Seed Admin User (in Postgres)
    let admin_email = "admin@enterprise.com";
    if let Ok(None) = metadata.get_user_by_email(admin_email).await {
        info!("Seeding default admin user...");
        let hash = auth_service.hash_password("password")?;
        metadata.create_user("admin_v1", admin_email, &hash, "admin").await?;
    }

    info!("All services connected successfully!");

    // Initialize Embedding Provider
    info!("Initializing Embedding Provider...");
    let provider_type = metadata.get_config("embedding_provider").await.ok().flatten().unwrap_or("mock".into());
    let model = metadata.get_config("embedding_model").await.ok().flatten().unwrap_or("default".into());
    
    let embedding_provider: Arc<dyn sembra_core::providers::EmbeddingProvider> = match provider_type.as_str() {
        "openai" => {
            let api_key = metadata.get_config("openai_api_key").await.ok().flatten()
                .unwrap_or_else(|| std::env::var("OPENAI_API_KEY").unwrap_or_default());
            info!("Using OpenAI Provider (model: {})", model);
            Arc::new(sembra_core::providers::openai::OpenAIProvider::new(api_key, model))
        },
        "ollama" => {
            let base_url = std::env::var("OLLAMA_BASE_URL").unwrap_or("http://localhost:11434".into());
            info!("Using Ollama Provider (model: {})", model);
            Arc::new(sembra_core::providers::ollama::OllamaProvider::new(base_url, model))
        },
        _ => {
            info!("Using Mock Provider");
            Arc::new(sembra_core::providers::mock::MockProvider::new())
        }
    };

    // Initialize LLM Provider
    info!("Initializing LLM Provider...");
    let llm_type = metadata.get_config("llm_provider").await.ok().flatten().unwrap_or("openai".into());
    let llm_model = metadata.get_config("llm_model").await.ok().flatten().unwrap_or("gpt-4o".into());

    let llm_provider: Arc<dyn sembra_core::providers::llm::LLMProvider> = match llm_type.as_str() {
        "ollama" => {
            let base_url = std::env::var("OLLAMA_BASE_URL").unwrap_or("http://localhost:11434".into());
            info!("Using Ollama LLM Provider ({})", llm_model);
            Arc::new(sembra_core::providers::llm::OllamaLLMProvider::new(base_url, llm_model))
        },
        _ => {
            let api_key = metadata.get_config("openai_api_key").await.ok().flatten()
                .unwrap_or_else(|| std::env::var("OPENAI_API_KEY").unwrap_or_default());
            info!("Using OpenAI LLM Provider ({})", llm_model);
            Arc::new(sembra_core::providers::llm::OpenAILLMProvider::new(api_key, llm_model))
        }
    };

    // Create application state
    let state = Arc::new(RwLock::new(AppState {
        cache,
        metadata,
        vector_db,
        graph_db,
        auth: auth_service,
        embedding_provider,
        llm_provider,
    }));

    // Create router
    let app = handlers::create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
