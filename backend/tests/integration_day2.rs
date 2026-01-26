//! Day 2 Integration Test: Barq-DB Storage Layer
//! 
//! This test requires a running PostgreSQL instance with pgvector.
//! Run with: DATABASE_URL=postgresql://postgres:password@localhost:5432/sembra cargo test

use sembra_storage::BarqDB;

/// Test that requires DATABASE_URL environment variable
/// Run with Docker: docker-compose up -d postgres && DATABASE_URL=... cargo test
#[tokio::test]
#[ignore] // Requires running PostgreSQL - run explicitly with --ignored
async fn test_barq_db_storage() {
    let db_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("⚠️ Skipping test - DATABASE_URL not set");
            return;
        }
    };

    let db = BarqDB::connect(&db_url, 5)
        .await
        .expect("Failed to connect to DB");

    // Insert 500 chunks
    for i in 0..500 {
        db.insert_chunk(
            &format!("chunk:{}", i),
            "doc:test",
            &format!("Test chunk {} with sample text content", i),
            vec![0.1; 768], // 768-dim embedding
            serde_json::json!({"index": i}),
        )
        .await
        .expect("Failed to insert chunk");
    }

    // Verify count
    let count = db
        .chunk_count()
        .await
        .expect("Failed to count chunks");

    assert!(count >= 500, "Expected at least 500 chunks, got {}", count);
    println!("✅ Day 2: Stored {} chunks in Barq-DB", count);
}

/// Test basic storage operations without requiring database
#[tokio::test]
async fn test_stored_chunk_serialization() {
    use sembra_storage::StoredChunk;
    
    let chunk = StoredChunk {
        chunk_id: "test:1".to_string(),
        document_id: "doc:1".to_string(),
        text: "Hello world".to_string(),
        metadata: Some(serde_json::json!({"key": "value"})),
        created_at: Some(1234567890),
    };
    
    let json = serde_json::to_string(&chunk).expect("Serialize");
    let parsed: StoredChunk = serde_json::from_str(&json).expect("Deserialize");
    
    assert_eq!(parsed.chunk_id, "test:1");
    assert_eq!(parsed.document_id, "doc:1");
}
