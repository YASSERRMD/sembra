//! SEMBRA Storage Layer - Hybrid Storage (Postgres + BarqDB)
//!
//! - `MetadataStore`: Postgres for Auth, Config, Tenants (SQL)
//! - `BarqDBClient`: BarqDB for Vector Storage (HTTP)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool};

// ==================== Postgres Metadata Store ====================

/// User entity stored in Postgres
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub role: String,
}

/// System configuration stored in Postgres
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

/// Metadata Store - Postgres for Auth, Config, Tenants
pub struct MetadataStore {
    pool: PgPool,
}

impl MetadataStore {
    /// Connect to PostgreSQL database
    pub async fn connect(database_url: &str, pool_size: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Initialize all schemas
    pub async fn init_schema(&self) -> Result<()> {
        // Users table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id VARCHAR(255) PRIMARY KEY,
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                role VARCHAR(50) NOT NULL,
                created_at BIGINT
            )"
        ).execute(&self.pool).await?;

        // Config table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS system_config (
                key VARCHAR(255) PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at BIGINT
            )"
        ).execute(&self.pool).await?;

        // Tenants table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tenants (
                id VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                api_key_hash TEXT,
                settings JSONB,
                created_at BIGINT
            )"
        ).execute(&self.pool).await?;

        Ok(())
    }

    // ---- User Operations ----

    pub async fn create_user(&self, id: &str, email: &str, hash: &str, role: &str) -> Result<()> {
        let ts = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, role, created_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id).bind(email).bind(hash).bind(role).bind(ts)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        sqlx::query_as::<_, User>("SELECT id, email, password_hash, role FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    // ---- Config Operations ----

    pub async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let ts = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO system_config (key, value, updated_at) VALUES ($1, $2, $3)
             ON CONFLICT(key) DO UPDATE SET value = EXCLUDED.value, updated_at = EXCLUDED.updated_at"
        )
        .bind(key).bind(value).bind(ts)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_config(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM system_config WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.0))
    }

    /// Health check
    pub async fn health(&self) -> Result<bool> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(true)
    }
}

// ==================== BarqDB Vector Store ====================

/// Stored chunk data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredChunk {
    pub id: u64,
    pub text: String,
    pub vector: Vec<f32>,
    pub payload: serde_json::Value,
}

/// Search result from BarqDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: u64,
    pub score: f32,
    pub payload: Option<serde_json::Value>,
}

/// BarqDB Client - connects to BarqDB vector database via REST API
pub struct BarqDBClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl BarqDBClient {
    pub fn new(base_url: &str, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        }
    }

    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("BARQ_DB_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let api_key = std::env::var("BARQ_DB_API_KEY").ok();
        Ok(Self::new(&base_url, api_key))
    }

    pub async fn create_collection(&self, name: &str, dimension: usize, metric: &str) -> Result<()> {
        let url = format!("{}/collections", self.base_url);
        let body = serde_json::json!({
            "name": name,
            "dimension": dimension,
            "metric": metric
        });

        let mut req = self.client.post(&url).json(&body);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() && resp.status().as_u16() != 409 {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create collection: {}", text);
        }
        Ok(())
    }

    pub async fn insert(&self, collection: &str, id: u64, vector: Vec<f32>, payload: serde_json::Value) -> Result<()> {
        let url = format!("{}/collections/{}/documents", self.base_url, collection);
        let body = serde_json::json!({ "id": id, "vector": vector, "payload": payload });

        let mut req = self.client.post(&url).json(&body);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to insert: {}", text);
        }
        Ok(())
    }

    pub async fn search(&self, collection: &str, vector: Vec<f32>, top_k: usize) -> Result<Vec<SearchResult>> {
        let url = format!("{}/collections/{}/search", self.base_url, collection);
        let body = serde_json::json!({ "vector": vector, "top_k": top_k });

        let mut req = self.client.post(&url).json(&body);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Search failed: {}", text);
        }
        Ok(resp.json().await?)
    }

    pub async fn hybrid_search(&self, collection: &str, vector: Vec<f32>, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        let url = format!("{}/collections/{}/hybrid_search", self.base_url, collection);
        let body = serde_json::json!({ "vector": vector, "query": query, "top_k": top_k });

        let mut req = self.client.post(&url).json(&body);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Hybrid search failed: {}", text);
        }
        Ok(resp.json().await?)
    }

    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.status().is_success())
    }
}
