//! Day 1 Integration Test: AiMesh Consumer + Celrix Cache

use sembra_cache::CelrixCache;
use sembra_core::aimesh::AiMeshConsumer;

#[tokio::test]
async fn test_aimesh_consumer_and_celrix_cache() {
    // Initialize consumer (mock)
    let consumer = AiMeshConsumer::connect_to_broker("localhost:9092")
        .await
        .expect("Failed to connect");

    // Initialize cache
    let cache = CelrixCache::new(100_000, 86400);

    // Read batch from consumer
    let batch = consumer
        .read_batch(100)
        .await
        .expect("Failed to read batch");

    assert_eq!(batch.len(), 100);

    // Cache embeddings
    for chunk in &batch {
        let embedding = vec![0.1, 0.2, 0.3]; // Mock embedding
        cache.insert(chunk.chunk_id.clone(), embedding).await;
    }

    // Verify cache hits
    for chunk in &batch {
        let cached = cache.get(&chunk.chunk_id).await;
        assert!(cached.is_some(), "Embedding not found in cache");
    }

    let hit_rate = cache.hit_rate().await;
    assert!(hit_rate > 95.0, "Hit rate too low: {}", hit_rate);

    println!("✅ Day 1: AiMesh + Celrix working");
}
