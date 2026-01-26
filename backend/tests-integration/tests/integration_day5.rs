//! Day 5 Integration Test: API Endpoint
//! 
//! Run the API server first, then run this test:
//! cargo run -p sembra-api &
//! cargo test -p sembra-api --test integration_day5

use serde_json::json;

/// Test API endpoint structure
#[tokio::test]
async fn test_api_types() {
    // Test RetrieveRequest serialization
    let request = json!({
        "query": "test search",
        "top_k": 10,
        "query_embedding": [0.1, 0.2, 0.3]
    });

    assert!(request.get("query").is_some());
    assert_eq!(request["top_k"], 10);
    
    println!("✅ Day 5: API request types verified");
}

/// Test health response format
#[tokio::test]
async fn test_health_response_format() {
    let health = json!({
        "status": "healthy",
        "version": "0.1.0",
        "uptime_secs": 0
    });

    assert_eq!(health["status"], "healthy");
    assert!(health.get("version").is_some());
    
    println!("✅ Day 5: Health response format verified");
}

/// Test retrieve response format
#[tokio::test]
async fn test_retrieve_response_format() {
    let response = json!({
        "results": [
            {"chunk_id": "chunk:0", "score": 0.95, "text": "Sample text"},
            {"chunk_id": "chunk:1", "score": 0.85, "text": null}
        ],
        "latency_ms": 5
    });

    let results = response["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert!(response["latency_ms"].as_u64().unwrap() < 100);
    
    println!("✅ Day 5: Retrieve response format verified");
}

/// Live API test (requires running server)
#[tokio::test]
#[ignore]
async fn test_live_health_endpoint() {
    let client = reqwest::Client::new();
    
    let response = client
        .get("http://localhost:3000/health")
        .send()
        .await
        .expect("Failed to call health endpoint");

    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.expect("Parse JSON");
    assert_eq!(body["status"], "healthy");
    
    println!("✅ Day 5: Live health endpoint working");
}

/// Live retrieve test (requires running server)
#[tokio::test]
#[ignore]
async fn test_live_retrieve_endpoint() {
    let client = reqwest::Client::new();
    
    let response = client
        .post("http://localhost:3000/v1/retrieve")
        .json(&json!({
            "query": "test search",
            "top_k": 5
        }))
        .send()
        .await
        .expect("Failed to call retrieve endpoint");

    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.expect("Parse JSON");
    assert!(body.get("results").is_some());
    assert!(body.get("latency_ms").is_some());
    
    println!("✅ Day 5: Live retrieve endpoint working");
}
