//! SEMBRA Storage Layer - Hybrid Storage (Postgres + BarqDB)
//!
//! - `MetadataStore`: Postgres for Auth, Config, Tenants, Documents (SQL)
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

        // Documents table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS documents (
                document_id VARCHAR(255) PRIMARY KEY,
                name VARCHAR(512) NOT NULL,
                chunk_count INTEGER NOT NULL DEFAULT 0,
                total_chars INTEGER NOT NULL DEFAULT 0,
                status VARCHAR(50) NOT NULL DEFAULT 'processing',
                created_at BIGINT NOT NULL
            )"
        ).execute(&self.pool).await?;

        // Document chunks mapping table (for deletion)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS document_chunks (
                id SERIAL PRIMARY KEY,
                document_id VARCHAR(255) NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
                chunk_id BIGINT NOT NULL,
                graph_node_id BIGINT
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

    // ---- Document Operations ----

    pub async fn insert_document(&self, document_id: &str, name: &str, chunk_count: i32, total_chars: i32) -> Result<()> {
        let ts = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO documents (document_id, name, chunk_count, total_chars, status, created_at)
             VALUES ($1, $2, $3, $4, 'ready', $5)
             ON CONFLICT(document_id) DO UPDATE SET name = EXCLUDED.name, chunk_count = EXCLUDED.chunk_count,
             total_chars = EXCLUDED.total_chars, status = EXCLUDED.status"
        )
        .bind(document_id).bind(name).bind(chunk_count).bind(total_chars).bind(ts)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn insert_document_chunk(&self, document_id: &str, chunk_id: u64, graph_node_id: Option<u64>) -> Result<()> {
        sqlx::query(
            "INSERT INTO document_chunks (document_id, chunk_id, graph_node_id) VALUES ($1, $2, $3)"
        )
        .bind(document_id).bind(chunk_id as i64).bind(graph_node_id.map(|id| id as i64))
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_documents(&self) -> Result<Vec<DocumentRecord>> {
        let docs = sqlx::query_as::<_, DocumentRecord>(
            "SELECT document_id, name, chunk_count, total_chars, status, created_at FROM documents ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(docs)
    }

    pub async fn get_document_chunk_ids(&self, document_id: &str) -> Result<Vec<(i64, Option<i64>)>> {
        let rows: Vec<(i64, Option<i64>)> = sqlx::query_as(
            "SELECT chunk_id, graph_node_id FROM document_chunks WHERE document_id = $1"
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete_document(&self, document_id: &str) -> Result<bool> {
        // Cascading delete will also remove document_chunks rows
        let result = sqlx::query("DELETE FROM documents WHERE document_id = $1")
            .bind(document_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Health check
    pub async fn health(&self) -> Result<bool> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(true)
    }
}

/// Document metadata stored in Postgres
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentRecord {
    pub document_id: String,
    pub name: String,
    pub chunk_count: i32,
    pub total_chars: i32,
    pub status: String,
    pub created_at: i64,
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

/// BarqDB raw search response format
#[derive(Debug, Deserialize)]
struct BarqSearchResponse {
    results: Vec<BarqSearchHit>,
}

#[derive(Debug, Deserialize)]
struct BarqSearchHit {
    id: BarqId,
    score: f32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum BarqId {
    Tagged { #[serde(rename = "U64")] u64_val: u64 },
    Plain(u64),
}

impl BarqId {
    fn value(&self) -> u64 {
        match self {
            BarqId::Tagged { u64_val } => *u64_val,
            BarqId::Plain(v) => *v,
        }
    }
}

/// BarqDB document response format
#[derive(Debug, Deserialize)]
struct BarqDocument {
    payload: Option<serde_json::Value>,
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

    /// Get a document by ID from a collection (includes payload)
    pub async fn get_document(&self, collection: &str, id: u64) -> Result<Option<serde_json::Value>> {
        let url = format!("{}/collections/{}/documents/{}", self.base_url, collection, id);
        // eprintln!("Fetching document: {}", url);

        let mut req = self.client.get(&url);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            eprintln!("get_document failed: status={}", status);
            return Ok(None);
        }
        
        let text = resp.text().await?;
        
        // Parse as generic Value
        let doc: serde_json::Value = match serde_json::from_str(&text) {
             Ok(d) => d,
             Err(e) => {
                 eprintln!("get_document failed deserialization: {} | Text: {}", e, text);
                 return Err(e.into());
             }
        };
        
        // Extract payload: Handle wrapping { "document": { "payload": ... } } or direct { "payload": ... }
        let payload_opt = if let Some(inner) = doc.get("document") {
            inner.get("payload")
        } else {
            doc.get("payload")
        };

        if let Some(payload) = payload_opt {
            Ok(Some(payload.clone()))
        } else {
            // Only log if we expected a payload but found none (and it's not a search result wrapper)
            // eprintln!("get_document json has no 'payload' field! JSON: {}", doc);
            Ok(None)
        }
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
        
        let raw: BarqSearchResponse = resp.json().await?;
        eprintln!("Search returned {} raw results", raw.results.len());
        
        // Fetch payloads for each result
        let mut results = Vec::new();
        for hit in raw.results {
            let id = hit.id.value();
            match self.get_document(collection, id).await {
                Ok(Some(payload)) => {
                    results.push(SearchResult {
                        id,
                        score: hit.score,
                        payload: Some(payload),
                    });
                },
                Ok(None) => {
                    eprintln!("get_document returned None for id {}", id);
                    results.push(SearchResult {
                        id,
                        score: hit.score,
                        payload: None,
                    });
                },
                Err(e) => {
                    eprintln!("get_document errored for id {}: {}", id, e);
                    // Decide whether to push with None or skip
                    // Current behavior was unwrap_or(None) which pushes with None
                    results.push(SearchResult {
                        id,
                        score: hit.score,
                        payload: None,
                    });
                }
            }
        }
        
        Ok(results)
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
        
        let raw: BarqSearchResponse = resp.json().await?;
        
        let mut results = Vec::new();
        for hit in raw.results {
            let id = hit.id.value();
            let payload = self.get_document(collection, id).await.unwrap_or(None);
            results.push(SearchResult {
                id,
                score: hit.score,
                payload,
            });
        }
        
        Ok(results)
    }

    pub async fn delete_document(&self, collection: &str, id: u64) -> Result<()> {
        let url = format!("{}/collections/{}/documents/{}", self.base_url, collection, id);
        
        let mut req = self.client.delete(&url);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to delete document {}: {} - {}", id, status, text);
        }
        Ok(())
    }

    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.status().is_success())
    }
}
