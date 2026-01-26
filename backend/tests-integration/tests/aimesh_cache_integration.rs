use sembra_core::AiMeshConsumer;
use sembra_cache::CelrixCache;

#[tokio::test]
async fn test_cache_insert_and_retrieve() {
    let cache = CelrixCache::new(100, 10);
    cache.insert("chunk:1".into(), vec![0.1, 0.2, 0.3]).await;
    
    let result = cache.get("chunk:1").await;
    assert!(result.is_some());
    assert_eq!(result.unwrap(), vec![0.1, 0.2, 0.3]);
}

#[tokio::test]
async fn test_aimesh_consumer_connect() {
    let consumer = AiMeshConsumer::connect_to_broker("localhost:9092")
        .await
        .expect("Failed to connect");
    assert_eq!(consumer.topic, "sembra:chunks");
}

#[tokio::test]
async fn test_aimesh_consumer_read_batch() {
    let consumer = AiMeshConsumer::connect_to_broker("localhost:9092")
        .await
        .unwrap();
    let batch = consumer.read_batch(5).await.unwrap();
    
    assert_eq!(batch.len(), 5);
    assert_eq!(batch[0].chunk_id, "chunk:0");
}
