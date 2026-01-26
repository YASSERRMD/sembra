use serde::Deserialize;
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub openai: OpenAIConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/sembra".to_string());
            
        let builder = Config::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("database.url", db_url)?
            .set_default("database.pool_size", 10)?
            .set_default("openai.api_key", "sk-mock-key")? // Default mock
            .set_default("openai.model", "text-embedding-3-small")?
            .add_source(Environment::with_prefix("SEMBRA").separator("__"));

        builder.build()?.try_deserialize()
    }
}
