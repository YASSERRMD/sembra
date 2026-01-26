use sembra_storage::{SearchResult, StoredChunk};
use sembra_graph::{GraphNode, GraphEdge};

#[test]
fn test_stored_chunk_serialization() {
    let chunk = StoredChunk {
        chunk_id: "chunk:test".to_string(),
        document_id: "doc:1".to_string(),
        text: "test content".to_string(),
        metadata: Some(serde_json::json!({"key": "value"})),
        created_at: Some(1234567890),
    };
    
    let json = serde_json::to_string(&chunk).unwrap();
    assert!(json.contains("chunk:test"));
    assert!(json.contains("doc:1"));
}

#[test]
fn test_search_result_serialization() {
    let result = SearchResult {
        chunk_id: "chunk:1".to_string(),
        document_id: "doc:1".to_string(),
        text: "sample text".to_string(),
        score: 0.95,
    };
    
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("0.95"));
}

#[test]
fn test_graph_node_creation() {
    let node = GraphNode {
        node_id: "chunk:1".to_string(),
        node_type: "Chunk".to_string(),
        properties: serde_json::json!({"document_id": "doc:1"}),
        created_at: 1234567890,
    };
    
    let json = serde_json::to_string(&node).unwrap();
    assert!(json.contains("Chunk"));
    assert!(json.contains("chunk:1"));
}

#[test]
fn test_graph_edge_creation() {
    let edge = GraphEdge {
        edge_id: "edge:1".to_string(),
        source_id: "chunk:1".to_string(),
        target_id: "chunk:2".to_string(),
        edge_type: "SIMILAR_TO".to_string(),
        weight: 0.92,
        properties: serde_json::json!({"similarity": 0.92}),
    };
    
    let json = serde_json::to_string(&edge).unwrap();
    assert!(json.contains("SIMILAR_TO"));
    assert!(json.contains("0.92"));
}

// Client test removed as GraphDB requires a live DB connection

