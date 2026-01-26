//! Barq-DB Storage Layer - HTTP Client for Barq-DB Vector Database
//!
//! Connects to Barq-DB REST API at http://localhost:8080

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Stored chunk data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredChunk {
    pub id: u64,
    pub text: String,
    pub vector: Vec<f32>,
    pub payload: serde_json::Value,
}

/// Search result from Barq-DB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: u64,
    pub score: f32,
    pub payload: Option<serde_json::Value>,
}

/// Barq-DB Client - connects to Barq-DB vector database via REST API
pub struct BarqDBClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl BarqDBClient {
    /// Create a new Barq-DB client
    pub fn new(base_url: &str, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        }
    }

    /// Create from environment variable
    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("BARQ_DB_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let api_key = std::env::var("BARQ_DB_API_KEY").ok();
        Ok(Self::new(&base_url, api_key))
    }

    /// Create a collection
    pub async fn create_collection(
        &self,
        name: &str,
        dimension: usize,
        metric: &str,
    ) -> Result<()> {
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
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create collection: {} - {}", status, text);
        }
        Ok(())
    }

    /// Insert a document into a collection
    pub async fn insert(
        &self,
        collection: &str,
        id: u64,
        vector: Vec<f32>,
        payload: serde_json::Value,
    ) -> Result<()> {
        let url = format!("{}/collections/{}/documents", self.base_url, collection);
        let body = serde_json::json!({
            "id": id,
            "vector": vector,
            "payload": payload
        });

        let mut req = self.client.post(&url).json(&body);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to insert document: {} - {}", status, text);
        }
        Ok(())
    }

    /// Vector search
    pub async fn search(
        &self,
        collection: &str,
        vector: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let url = format!("{}/collections/{}/search", self.base_url, collection);
        let body = serde_json::json!({
            "vector": vector,
            "top_k": top_k
        });

        let mut req = self.client.post(&url).json(&body);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Search failed: {} - {}", status, text);
        }

        let results: Vec<SearchResult> = resp.json().await?;
        Ok(results)
    }

    /// Hybrid search (vector + keyword)
    pub async fn hybrid_search(
        &self,
        collection: &str,
        vector: Vec<f32>,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let url = format!("{}/collections/{}/hybrid_search", self.base_url, collection);
        let body = serde_json::json!({
            "vector": vector,
            "query": query,
            "top_k": top_k
        });

        let mut req = self.client.post(&url).json(&body);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Hybrid search failed: {} - {}", status, text);
        }

        let results: Vec<SearchResult> = resp.json().await?;
        Ok(results)
    }

    /// Health check
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_chunk_serialization() {
        let chunk = StoredChunk {
            id: 1,
            text: "Hello world".to_string(),
            vector: vec![0.1, 0.2, 0.3],
            payload: serde_json::json!({"key": "value"}),
        };

        let json = serde_json::to_string(&chunk).expect("Serialize");
        let parsed: StoredChunk = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(parsed.id, 1);
        assert_eq!(parsed.text, "Hello world");
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            id: 1,
            score: 0.95,
            payload: Some(serde_json::json!({"name": "test"})),
        };

        let json = serde_json::to_string(&result).expect("Serialize");
        let parsed: SearchResult = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(parsed.id, 1);
        assert_eq!(parsed.score, 0.95);
    }

    #[test]
    fn test_client_creation() {
        let client = BarqDBClient::new("http://localhost:8080", None);
        assert_eq!(client.base_url, "http://localhost:8080");
    }
}
