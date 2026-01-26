//! Barq-GraphDB - Graph storage layer for document relationships

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool, Row};

/// A node in the graph
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GraphNode {
    pub node_id: String,
    pub node_type: String,
    pub properties: serde_json::Value,
    pub created_at: i64,
}

/// An edge between nodes
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GraphEdge {
    pub edge_id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub weight: f32,
    pub properties: serde_json::Value,
}

/// Barq-GraphDB - Graph database for document relationships
pub struct GraphDB {
    pool: PgPool,
}

impl GraphDB {
    /// Connect to PostgreSQL database
    pub async fn connect(database_url: &str, pool_size: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Initialize graph tables
    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS graph_nodes (
                node_id VARCHAR(255) PRIMARY KEY,
                node_type VARCHAR(100) NOT NULL,
                properties JSONB,
                created_at BIGINT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS graph_edges (
                edge_id VARCHAR(255) PRIMARY KEY,
                source_id VARCHAR(255) NOT NULL REFERENCES graph_nodes(node_id) ON DELETE CASCADE,
                target_id VARCHAR(255) NOT NULL REFERENCES graph_nodes(node_id) ON DELETE CASCADE,
                edge_type VARCHAR(100) NOT NULL,
                weight REAL DEFAULT 1.0,
                properties JSONB
            );
            
            CREATE INDEX IF NOT EXISTS idx_edges_source ON graph_edges(source_id);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON graph_edges(target_id);
            CREATE INDEX IF NOT EXISTS idx_edges_type ON graph_edges(edge_type);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Add a node to the graph
    pub async fn add_node(
        &self,
        node_id: &str,
        node_type: &str,
        properties: serde_json::Value,
    ) -> Result<()> {
        let created_at = chrono::Utc::now().timestamp_millis();

        sqlx::query(
            r#"
            INSERT INTO graph_nodes (node_id, node_type, properties, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(node_id) DO UPDATE SET
                node_type = EXCLUDED.node_type,
                properties = EXCLUDED.properties
            "#,
        )
        .bind(node_id)
        .bind(node_type)
        .bind(properties)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a node by ID
    pub async fn get_node(&self, node_id: &str) -> Result<Option<GraphNode>> {
        let node = sqlx::query_as::<_, GraphNode>(
            "SELECT node_id, node_type, properties, created_at FROM graph_nodes WHERE node_id = $1",
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(node)
    }

    /// Create an edge between two nodes
    pub async fn link_nodes(
        &self,
        source_id: &str,
        target_id: &str,
        edge_type: &str,
        weight: f32,
        properties: serde_json::Value,
    ) -> Result<String> {
        let edge_id = format!("{}->{}:{}", source_id, target_id, edge_type);

        sqlx::query(
            r#"
            INSERT INTO graph_edges (edge_id, source_id, target_id, edge_type, weight, properties)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT(edge_id) DO UPDATE SET
                weight = EXCLUDED.weight,
                properties = EXCLUDED.properties
            "#,
        )
        .bind(&edge_id)
        .bind(source_id)
        .bind(target_id)
        .bind(edge_type)
        .bind(weight)
        .bind(properties)
        .execute(&self.pool)
        .await?;

        Ok(edge_id)
    }
    
    /// Get neighbors of a node
    pub async fn get_neighbors(
        &self,
        node_id: &str,
    ) -> Result<Vec<(String, f32)>> {
        let rows = sqlx::query(
            r#"
            SELECT target_id, weight FROM graph_edges 
            WHERE source_id = $1
            ORDER BY weight DESC
            "#,
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| (r.get::<String, _>("target_id"), r.get::<f32, _>("weight")))
            .collect())
    }

    pub async fn health_check(&self) -> Result<bool> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(true)
    }
}
