//! Barq-GraphDB Client - HTTP Client for Barq-GraphDB Hybrid Graph+Vector Database
//!
//! Connects to Barq-GraphDB REST API at http://localhost:8081

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// A node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: u64,
    pub label: String,
    pub properties: serde_json::Value,
}

/// An edge between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: u64,
    pub target: u64,
    pub edge_type: String,
    pub weight: Option<f32>,
}

/// Hybrid query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridResult {
    pub id: u64,
    pub score: f32,
    pub path: Vec<u64>,
}

/// Barq-GraphDB Client - connects to Barq-GraphDB via REST API
pub struct BarqGraphDBClient {
    client: reqwest::Client,
    base_url: String,
}

impl BarqGraphDBClient {
    /// Create a new Barq-GraphDB client
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Create from environment variable
    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("BARQ_GRAPHDB_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());
        Ok(Self::new(&base_url))
    }

    /// Add a node to the graph
    pub async fn add_node(&self, id: u64, label: &str, properties: serde_json::Value) -> Result<()> {
        let url = format!("{}/nodes", self.base_url);
        let body = serde_json::json!({
            "id": id,
            "label": label,
            "properties": properties
        });

        let resp = self.client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to add node: {} - {}", status, text);
        }
        Ok(())
    }

    /// Add an edge between nodes
    pub async fn add_edge(&self, source: u64, target: u64, edge_type: &str) -> Result<()> {
        let url = format!("{}/edges", self.base_url);
        let body = serde_json::json!({
            "from": source,
            "to": target,
            "edge_type": edge_type
        });

        let resp = self.client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to add edge: {} - {}", status, text);
        }
        Ok(())
    }

    /// Set embedding for a node
    pub async fn set_embedding(&self, id: u64, embedding: Vec<f32>) -> Result<()> {
        let url = format!("{}/nodes/{}/embedding", self.base_url, id);
        let body = serde_json::json!({
            "embedding": embedding
        });

        let resp = self.client.put(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to set embedding: {} - {}", status, text);
        }
        Ok(())
    }

    /// Get neighbors of a node
    pub async fn get_neighbors(&self, id: u64) -> Result<Vec<u64>> {
        let url = format!("{}/nodes/{}/neighbors", self.base_url, id);
        let resp = self.client.get(&url).send().await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get neighbors: {} - {}", status, text);
        }

        let neighbors: Vec<u64> = resp.json().await?;
        Ok(neighbors)
    }

    /// Hybrid query combining vector similarity and graph distance
    pub async fn hybrid_query(
        &self,
        embedding: Vec<f32>,
        start_node: u64,
        max_depth: usize,
        top_k: usize,
        vector_weight: f32,
        graph_weight: f32,
    ) -> Result<Vec<HybridResult>> {
        let url = format!("{}/hybrid_query", self.base_url);
        let body = serde_json::json!({
            "embedding": embedding,
            "start_node": start_node,
            "max_depth": max_depth,
            "top_k": top_k,
            "vector_weight": vector_weight,
            "graph_weight": graph_weight
        });

        let resp = self.client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Hybrid query failed: {} - {}", status, text);
        }

        let results: Vec<HybridResult> = resp.json().await?;
        Ok(results)
    }

    /// Get all nodes with a given label
    pub async fn get_nodes_by_label(&self, label: &str) -> Result<Vec<GraphNode>> {
        let url = format!("{}/nodes?label={}", self.base_url, label);
        let resp = self.client.get(&url).send().await?;
        
        if !resp.status().is_success() {
            // If the endpoint doesn't exist or returns error, return empty
            return Ok(vec![]);
        }

        let nodes: Vec<GraphNode> = resp.json().await.unwrap_or_default();
        Ok(nodes)
    }

    /// Delete a node from the graph
    pub async fn delete_node(&self, id: u64) -> Result<()> {
        let url = format!("{}/nodes/{}", self.base_url, id);
        let resp = self.client.delete(&url).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to delete node {}: {} - {}", id, status, text);
        }
        Ok(())
    }

    /// Health check
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.status().is_success())
    }
}
