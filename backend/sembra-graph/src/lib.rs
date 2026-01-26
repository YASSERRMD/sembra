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

    /// Delete a node and its edges
    pub async fn delete_node(&self, node_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM graph_nodes WHERE node_id = $1")
            .bind(node_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
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
        edge_type: Option<&str>,
    ) -> Result<Vec<(String, f32)>> {
        let query = match edge_type {
            Some(et) => sqlx::query(
                r#"
                SELECT target_id, weight FROM graph_edges 
                WHERE source_id = $1 AND edge_type = $2
                ORDER BY weight DESC
                "#,
            )
            .bind(node_id)
            .bind(et),
            None => sqlx::query(
                r#"
                SELECT target_id, weight FROM graph_edges 
                WHERE source_id = $1
                ORDER BY weight DESC
                "#,
            )
            .bind(node_id),
        };

        let rows = query.fetch_all(&self.pool).await?;

        Ok(rows
            .iter()
            .map(|r| (r.get::<String, _>("target_id"), r.get::<f32, _>("weight")))
            .collect())
    }

    /// Get node count
    pub async fn node_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM graph_nodes")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("count"))
    }

    /// Get edge count
    pub async fn edge_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM graph_edges")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("count"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_node_serialization() {
        let node = GraphNode {
            node_id: "node:1".to_string(),
            node_type: "document".to_string(),
            properties: serde_json::json!({"title": "Test"}),
            created_at: 1234567890,
        };

        let json = serde_json::to_string(&node).expect("Serialize");
        let parsed: GraphNode = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(parsed.node_id, "node:1");
        assert_eq!(parsed.node_type, "document");
    }

    #[test]
    fn test_graph_edge_serialization() {
        let edge = GraphEdge {
            edge_id: "e1".to_string(),
            source_id: "node:1".to_string(),
            target_id: "node:2".to_string(),
            edge_type: "references".to_string(),
            weight: 0.85,
            properties: serde_json::json!({}),
        };

        let json = serde_json::to_string(&edge).expect("Serialize");
        let parsed: GraphEdge = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(parsed.source_id, "node:1");
        assert_eq!(parsed.target_id, "node:2");
        assert_eq!(parsed.weight, 0.85);
    }
}
