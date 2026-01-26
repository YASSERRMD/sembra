//! Barq-GraphDB Client - HTTP client for graph database operations

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Graph node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub properties: serde_json::Value,
}

/// Graph edge representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from_id: String,
    pub to_id: String,
    pub relation: String,
    pub properties: Option<serde_json::Value>,
}

/// Barq-GraphDB HTTP client
pub struct BarqGraphDB {
    client: Client,
    base_url: String,
}

impl BarqGraphDB {
    /// Create a new GraphDB client
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Add a node to the graph
    pub async fn add_node(&self, node: &GraphNode) -> Result<()> {
        self.client
            .post(format!("{}/nodes", self.base_url))
            .json(node)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Get a node by ID
    pub async fn get_node(&self, id: &str) -> Result<Option<GraphNode>> {
        let response = self.client
            .get(format!("{}/nodes/{}", self.base_url, id))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(Some(response.json().await?))
        } else if response.status().as_u16() == 404 {
            Ok(None)
        } else {
            response.error_for_status()?;
            Ok(None)
        }
    }

    /// Delete a node by ID
    pub async fn delete_node(&self, id: &str) -> Result<bool> {
        let response = self.client
            .delete(format!("{}/nodes/{}", self.base_url, id))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Create an edge between two nodes
    pub async fn add_edge(&self, edge: &GraphEdge) -> Result<()> {
        self.client
            .post(format!("{}/edges", self.base_url))
            .json(edge)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Get neighbors of a node
    pub async fn get_neighbors(&self, id: &str, relation: Option<&str>) -> Result<Vec<GraphNode>> {
        let mut url = format!("{}/nodes/{}/neighbors", self.base_url, id);
        if let Some(rel) = relation {
            url = format!("{}?relation={}", url, rel);
        }

        let response = self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    /// Create a chunk node with standard properties
    pub async fn create_chunk_node(&self, chunk_id: &str, document_id: &str) -> Result<()> {
        let node = GraphNode {
            id: chunk_id.to_string(),
            label: "Chunk".to_string(),
            properties: serde_json::json!({
                "document_id": document_id,
                "created_at": chrono::Utc::now().timestamp_millis()
            }),
        };
        self.add_node(&node).await
    }

    /// Link two chunks as similar
    pub async fn link_similar(&self, chunk_a: &str, chunk_b: &str, similarity: f32) -> Result<()> {
        let edge = GraphEdge {
            from_id: chunk_a.to_string(),
            to_id: chunk_b.to_string(),
            relation: "SIMILAR_TO".to_string(),
            properties: Some(serde_json::json!({ "similarity": similarity })),
        };
        self.add_edge(&edge).await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        let response = self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;
        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_node_serialization() {
        let node = GraphNode {
            id: "chunk:1".to_string(),
            label: "Chunk".to_string(),
            properties: serde_json::json!({"doc": "test"}),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("chunk:1"));
    }

    #[test]
    fn test_graph_edge_serialization() {
        let edge = GraphEdge {
            from_id: "chunk:1".to_string(),
            to_id: "chunk:2".to_string(),
            relation: "SIMILAR_TO".to_string(),
            properties: Some(serde_json::json!({"similarity": 0.95})),
        };
        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("SIMILAR_TO"));
    }
}
