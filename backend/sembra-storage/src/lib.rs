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

    /// Vector similarity search using pgvector cosine distance
    pub async fn vector_search(
        &self,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<(String, f32)>> {
        let embedding_vec = Vector::from(query_embedding.to_vec());
        
        let rows = sqlx::query(
            r#"
            SELECT chunk_id, 1 - (embedding <=> $1) as score
            FROM sembra_chunks
            WHERE embedding IS NOT NULL
            ORDER BY embedding <=> $1
            LIMIT $2
            "#,
        )
        .bind(&embedding_vec)
        .bind(top_k as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| (r.get::<String, _>("chunk_id"), r.get::<f32, _>("score")))
            .collect())
    }

    /// BM25 full-text search using PostgreSQL's built-in text search
    pub async fn bm25_search(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<(String, f32)>> {
        let rows = sqlx::query(
            r#"
            SELECT chunk_id, 
                   ts_rank_cd(to_tsvector('english', text), plainto_tsquery('english', $1)) as score
            FROM sembra_chunks
            WHERE to_tsvector('english', text) @@ plainto_tsquery('english', $1)
            ORDER BY score DESC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(top_k as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| (r.get::<String, _>("chunk_id"), r.get::<f32, _>("score")))
            .collect())
    }

    /// Hybrid search combining vector and BM25 with Reciprocal Rank Fusion (RRF)
    pub async fn hybrid_search(
        &self,
        query: &str,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<(String, f32)>> {
        // Get results from both search methods
        let bm25_results = self.bm25_search(query, 100).await?;
        let vector_results = self.vector_search(query_embedding, 100).await?;

        // RRF Fusion with k=60 (standard constant)
        let mut fused: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        
        // Weight for BM25 results
        for (rank, (chunk_id, _)) in bm25_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f32) * 0.5;
            *fused.entry(chunk_id.clone()).or_insert(0.0) += rrf_score;
        }
        
        // Weight for vector results
        for (rank, (chunk_id, _)) in vector_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f32) * 0.5;
            *fused.entry(chunk_id.clone()).or_insert(0.0) += rrf_score;
        }

        // Sort by fused score
        let mut sorted: Vec<_> = fused.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(sorted.into_iter().take(top_k).collect())
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require a running PostgreSQL instance
    // Run with: DATABASE_URL=... cargo test -p sembra-storage
}
