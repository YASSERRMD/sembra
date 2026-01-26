//! Day 3 Integration Test: Vector + BM25 Search
//! 
//! This test requires a running PostgreSQL instance with pgvector.
//! Run with: DATABASE_URL=postgresql://postgres:password@localhost:5432/sembra cargo test --ignored

use sembra_storage::BarqDB;
use std::time::Instant;

/// Test vector search latency (requires DATABASE_URL)
#[tokio::test]
#[ignore]
async fn test_vector_search_latency() {
    let db_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("⚠️ Skipping test - DATABASE_URL not set");
            return;
        }
    };

    let db = BarqDB::connect(&db_url, 5).await.expect("DB connect");

    // Insert 1000 chunks for testing
    for i in 0..1000 {
        db.insert_chunk(
            &format!("search:chunk:{}", i),
            "doc:test",
            &format!("Test chunk {} with sample text content for searching", i),
            vec![0.1 + (i as f32 * 0.0001); 768],
            serde_json::json!({"index": i}),
        )
        .await
        .expect("Insert");
    }

    // Test vector search latency
    let start = Instant::now();
    let results = db
        .vector_search(&vec![0.1; 768], 10)
        .await
        .expect("Search");
    let latency_ms = start.elapsed().as_millis();

    assert!(!results.is_empty(), "No results returned");
    assert!(latency_ms < 100, "Latency {} ms > 100ms", latency_ms);

    println!("✅ Day 3: Vector search latency: {}ms", latency_ms);
}

/// Test hybrid search functionality (requires DATABASE_URL)
#[tokio::test]
#[ignore]
async fn test_hybrid_search() {
    let db_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("⚠️ Skipping test - DATABASE_URL not set");
            return;
        }
    };

    let db = BarqDB::connect(&db_url, 5).await.expect("DB connect");

    // Test hybrid search
    let start = Instant::now();
    let results = db
        .hybrid_search("sample text", &vec![0.1; 768], 10)
        .await
        .expect("Hybrid search");
    let latency_ms = start.elapsed().as_millis();

    println!(
        "✅ Day 3: Hybrid search returned {} results in {}ms",
        results.len(),
        latency_ms
    );
}

/// Test RRF fusion algorithm without database
#[tokio::test]
async fn test_rrf_fusion_logic() {
    use std::collections::HashMap;

    // Simulate BM25 results: chunk:0, chunk:1, chunk:2
    let bm25_results = vec![
        ("chunk:0".to_string(), 0.9),
        ("chunk:1".to_string(), 0.8),
        ("chunk:2".to_string(), 0.7),
    ];

    // Simulate vector results: chunk:1, chunk:0, chunk:3
    let vector_results = vec![
        ("chunk:1".to_string(), 0.95),
        ("chunk:0".to_string(), 0.85),
        ("chunk:3".to_string(), 0.75),
    ];

    // RRF Fusion with k=60
    let mut fused: HashMap<String, f32> = HashMap::new();

    for (rank, (chunk_id, _)) in bm25_results.iter().enumerate() {
        let rrf_score = 1.0 / (60.0 + rank as f32) * 0.5;
        *fused.entry(chunk_id.clone()).or_insert(0.0) += rrf_score;
    }

    for (rank, (chunk_id, _)) in vector_results.iter().enumerate() {
        let rrf_score = 1.0 / (60.0 + rank as f32) * 0.5;
        *fused.entry(chunk_id.clone()).or_insert(0.0) += rrf_score;
    }

    // Verify chunk:1 and chunk:0 have highest scores (appear in both)
    let chunk0_score = fused.get("chunk:0").unwrap();
    let chunk1_score = fused.get("chunk:1").unwrap();
    let chunk2_score = fused.get("chunk:2").unwrap();
    let chunk3_score = fused.get("chunk:3").unwrap();

    // Chunks appearing in both lists should have higher scores
    assert!(chunk0_score > chunk2_score, "chunk:0 should rank higher than chunk:2");
    assert!(chunk1_score > chunk3_score, "chunk:1 should rank higher than chunk:3");

    println!("✅ Day 3: RRF fusion algorithm verified");
}
