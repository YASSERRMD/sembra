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
}

#[cfg(test)]
mod tests {
    // Integration tests require a running PostgreSQL instance
    // Run with: DATABASE_URL=... cargo test -p sembra-storage
}
