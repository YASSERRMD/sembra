//! Celrix Cache - high-performance embedding cache using moka

use moka::future::Cache;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// High-performance cache for storing embeddings
pub struct CelrixCache {
    cache: Cache<String, Vec<f32>>,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

impl CelrixCache {
    /// Create a new cache with specified capacity and TTL
    pub fn new(capacity: u64, ttl_secs: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(capacity)
                .time_to_live(Duration::from_secs(ttl_secs))
                .build(),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Insert an embedding into the cache
    pub async fn insert(&self, chunk_id: String, embedding: Vec<f32>) {
        self.cache.insert(chunk_id, embedding).await;
    }

    /// Get an embedding from the cache
    pub async fn get(&self, chunk_id: &str) -> Option<Vec<f32>> {
        match self.cache.get(chunk_id).await {
            Some(emb) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(emb)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Get the current hit rate as a percentage
    pub async fn hit_rate(&self) -> f32 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            (hits as f32 / total as f32) * 100.0
        }
    }

    /// Get the number of cache hits
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Get the number of cache misses
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_get() {
        let cache = CelrixCache::new(100, 3600);
        let embedding = vec![0.1, 0.2, 0.3];
        
        cache.insert("chunk:1".to_string(), embedding.clone()).await;
        
        let result = cache.get("chunk:1").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap(), embedding);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = CelrixCache::new(100, 3600);
        
        let result = cache.get("nonexistent").await;
        assert!(result.is_none());
        assert_eq!(cache.misses(), 1);
    }

    #[tokio::test]
    async fn test_hit_rate() {
        let cache = CelrixCache::new(100, 3600);
        
        // Insert some embeddings
        for i in 0..10 {
            cache.insert(format!("chunk:{}", i), vec![0.1, 0.2]).await;
        }
        
        // Hit them all
        for i in 0..10 {
            let _ = cache.get(&format!("chunk:{}", i)).await;
        }
        
        let hit_rate = cache.hit_rate().await;
        assert!(hit_rate > 99.0, "Expected 100% hit rate, got {}%", hit_rate);
    }
}
