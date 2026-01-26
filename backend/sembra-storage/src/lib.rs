//! Barq-DB Storage Layer - PostgreSQL with pgvector

use anyhow::Result;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool, Row};

/// Stored chunk data
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoredChunk {
    pub chunk_id: String,
    pub document_id: String,
    pub text: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<i64>,
}

/// Search result with score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub text: String,
    pub score: f32,
}

/// Barq-DB - High-performance PostgreSQL storage with pgvector
pub struct BarqDB {
    pool: PgPool,
}

impl BarqDB {
    /// Connect to PostgreSQL database
    pub async fn connect(database_url: &str, pool_size: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Insert a chunk with embedding into the database
    pub async fn insert_chunk(
        &self,
        chunk_id: &str,
        document_id: &str,
        text: &str,
        embedding: Vec<f32>,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let embedding_vec = Vector::from(embedding);
        let created_at = chrono::Utc::now().timestamp_millis();

        sqlx::query(
            r#"
            INSERT INTO sembra_chunks (chunk_id, document_id, text, embedding, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT(chunk_id) DO UPDATE SET
                text = EXCLUDED.text,
                embedding = EXCLUDED.embedding,
                metadata = EXCLUDED.metadata
            "#,
        )
        .bind(chunk_id)
        .bind(document_id)
        .bind(text)
        .bind(embedding_vec)
        .bind(metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get the total number of chunks in the database
    pub async fn chunk_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM sembra_chunks")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Get a chunk by ID
    pub async fn get_chunk(&self, chunk_id: &str) -> Result<Option<StoredChunk>> {
        let chunk = sqlx::query_as::<_, StoredChunk>(
            "SELECT chunk_id, document_id, text, metadata, created_at FROM sembra_chunks WHERE chunk_id = $1",
        )
        .bind(chunk_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chunk)
    }

    /// Delete a chunk by ID
    pub async fn delete_chunk(&self, chunk_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM sembra_chunks WHERE chunk_id = $1")
            .bind(chunk_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get pool reference for advanced operations
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Vector similarity search using pgvector cosine distance
    pub async fn vector_search(
        &self,
        query_embedding: Vec<f32>,
        limit: i32,
    ) -> Result<Vec<SearchResult>> {
        let query_vec = Vector::from(query_embedding);

        let rows = sqlx::query(
            r#"
            SELECT chunk_id, document_id, text, metadata,
                   1 - (embedding <=> $1) as score
            FROM sembra_chunks
            ORDER BY embedding <=> $1
            LIMIT $2
            "#,
        )
        .bind(&query_vec)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .iter()
            .map(|row| SearchResult {
                chunk_id: row.get("chunk_id"),
                document_id: row.get("document_id"),
                text: row.get("text"),
                score: row.get("score"),
            })
            .collect();

        Ok(results)
    }

    /// BM25 text search using PostgreSQL full-text search
    pub async fn bm25_search(&self, query: &str, limit: i32) -> Result<Vec<SearchResult>> {
        let rows = sqlx::query(
            r#"
            SELECT chunk_id, document_id, text, metadata,
                   ts_rank(to_tsvector('english', text), plainto_tsquery('english', $1)) as score
            FROM sembra_chunks
            WHERE to_tsvector('english', text) @@ plainto_tsquery('english', $1)
            ORDER BY score DESC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .iter()
            .map(|row| SearchResult {
                chunk_id: row.get("chunk_id"),
                document_id: row.get("document_id"),
                text: row.get("text"),
                score: row.get("score"),
            })
            .collect();

        Ok(results)
    }

    /// Hybrid search combining vector and BM25 with RRF fusion
    pub async fn hybrid_search(
        &self,
        query_embedding: Vec<f32>,
        query_text: &str,
        limit: i32,
        k: f32,
    ) -> Result<Vec<SearchResult>> {
        // Get results from both search methods
        let vector_results = self.vector_search(query_embedding, limit * 2).await?;
        let bm25_results = self.bm25_search(query_text, limit * 2).await?;

        // RRF fusion: score = sum(1 / (k + rank))
        let mut scores: std::collections::HashMap<String, (f32, SearchResult)> =
            std::collections::HashMap::new();

        for (rank, result) in vector_results.into_iter().enumerate() {
            let rrf_score = 1.0 / (k + rank as f32 + 1.0);
            let chunk_id = result.chunk_id.clone();
            scores
                .entry(chunk_id)
                .or_insert((0.0, result))
                .0 += rrf_score;
        }

        for (rank, result) in bm25_results.into_iter().enumerate() {
            let rrf_score = 1.0 / (k + rank as f32 + 1.0);
            scores
                .entry(result.chunk_id.clone())
                .and_modify(|(score, _)| *score += rrf_score)
                .or_insert((rrf_score, result));
        }

        let mut results: Vec<SearchResult> = scores
            .into_iter()
            .map(|(_, (score, mut result))| {
                result.score = score;
                result
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit as usize);

        Ok(results)
    }

    /// Initialize configuration table
    pub async fn init_config_table(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS system_config (
                key VARCHAR(255) PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at BIGINT
            )"
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Initialize authentication schema
    pub async fn init_auth_schema(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id VARCHAR(255) PRIMARY KEY,
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                role VARCHAR(50) NOT NULL,
                created_at BIGINT
            )"
        ).execute(&self.pool).await?;
        Ok(())
    }

    /// Set a configuration value
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

    /// Get a configuration value
    pub async fn get_config(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM system_config WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.0))
    }

    /// Create a new user
    pub async fn create_user(&self, id: &str, email: &str, hash: &str, role: &str) -> Result<()> {
        let ts = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, role, created_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id).bind(email).bind(hash).bind(role).bind(ts)
        .execute(&self.pool).await?;
        Ok(())
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        sqlx::query_as::<_, User>("SELECT id, email, password_hash, role FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
}

/// User Entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub role: String,
}

#[cfg(test)]
mod tests {
    // Integration tests require a running PostgreSQL instance
    // Run with: DATABASE_URL=... cargo test -p sembra-storage
}
