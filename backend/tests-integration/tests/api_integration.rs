use sembra_types::{RetrieveRequest, RetrieveResponse};
// use reqwest::{Client, StatusCode};

#[tokio::test]
async fn test_api_types() {
    let req = RetrieveRequest {
        query: "test query".to_string(),
        top_k: Some(5),
        query_embedding: vec![0.1; 768],
    };
    
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("test query"));
}

#[tokio::test]
async fn test_retrieve_response_format() {
    let resp = RetrieveResponse {
        results: vec![],
        latency_ms: 10,
    };
    
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("latency_ms"));
}

// Note: Real API tests require running services (Day 5 Goal)
// For now we test data structures and client interactions
#[test]
fn test_health_response_format() {
    use sembra_types::HealthResponse;
    let resp = HealthResponse {
        status: "healthy".to_string(),
        version: "0.1.0".to_string(),
        uptime_secs: 100,
    };
    assert_eq!(resp.status, "healthy");
}
