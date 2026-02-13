//! Upload handlers for document ingestion

use axum::{
    extract::{State, Multipart, multipart::Field},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::info;

use crate::state::AppState;
use crate::extractors;
use crate::chunking::{self, Chunk};

/// Upload response
#[derive(Serialize)]
pub struct UploadResponse {
    pub document_id: String,
    pub document_name: String,
    pub total_chars: usize,
    pub chunk_count: usize,
    pub status: String,
}

/// Error response
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

fn hash_to_u64(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (status, Json(ErrorResponse { error: msg.into() }))
}

/// Parse a single field from the multipart
async fn read_field_bytes(field: Field<'_>) -> (String, String, Vec<u8>) {
    let name = field.name().unwrap_or("").to_string();
    let file_name = field.file_name().unwrap_or("").to_string();
    let bytes = field.bytes().await.unwrap_or_default().to_vec();
    (name, file_name, bytes)
}

/// Handle file upload
pub async fn upload_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut document_id = String::new();
    let mut document_name = String::new();
    let mut chunk_size: usize = 512;
    let mut chunk_overlap: usize = 128;
    let mut file_data: Option<(String, Vec<u8>)> = None;
    
    // Parse multipart fields
    while let Ok(Some(field)) = multipart.next_field().await {
        let (name, file_name, bytes) = read_field_bytes(field).await;
        
        match name.as_str() {
            "document_id" => {
                document_id = String::from_utf8_lossy(&bytes).to_string();
            }
            "document_name" => {
                document_name = String::from_utf8_lossy(&bytes).to_string();
            }
            "chunk_size" => {
                let text = String::from_utf8_lossy(&bytes);
                chunk_size = text.parse().unwrap_or(512);
            }
            "chunk_overlap" => {
                let text = String::from_utf8_lossy(&bytes);
                chunk_overlap = text.parse().unwrap_or(128);
            }
            "file" | "files" | "files[]" => {
                if bytes.len() > 100 * 1024 * 1024 {
                    return Err(err(StatusCode::PAYLOAD_TOO_LARGE, "File exceeds 100MB limit"));
                }
                let fname = if file_name.is_empty() { "unknown.txt".to_string() } else { file_name };
                file_data = Some((fname, bytes));
            }
            _ => {}
        }
    }
    
    // Validate we have a file
    let (filename, data) = file_data.ok_or_else(|| err(StatusCode::BAD_REQUEST, "No file provided"))?;
    
    // Generate document_id if not provided
    let document_id = if document_id.is_empty() {
        format!("doc:{}", uuid::Uuid::new_v4())
    } else {
        document_id
    };
    
    // Use document name from form or filename
    let document_name = if document_name.is_empty() {
        filename.clone()
    } else {
        document_name
    };
    
    info!("Processing upload: {} ({} bytes)", filename, data.len());
    
    // Extract text
    let text = extractors::extract_from_bytes(&data, &filename)
        .map_err(|e| err(StatusCode::UNPROCESSABLE_ENTITY, format!("Extraction failed: {}", e)))?;
    
    let total_chars = text.len();
    info!("Extracted {} characters from {}", total_chars, filename);
    
    // Chunk the text
    let chunks: Vec<Chunk> = chunking::chunk_parallel(&text, &document_id, chunk_size, chunk_overlap).await;
    let chunk_count = chunks.len();
    info!("Created {} chunks (size={}, overlap={})", chunk_count, chunk_size, chunk_overlap);
    
    // Generate real embeddings for all chunks using the configured embedding provider
    let state_guard = state.read().await;
    
    let chunk_texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
    let embeddings = state_guard.embedding_provider.embed_documents(&chunk_texts).await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("Embedding generation failed: {}", e)))?;
    
    let embed_dim = embeddings.first().map(|v| v.len()).unwrap_or(384);
    info!("Generated {} embeddings (dim={})", embeddings.len(), embed_dim);
    
    // Ensure collection exists in BarqDB with the correct dimension
    if let Err(e) = state_guard.vector_db.create_collection("sembra_chunks", embed_dim, "Cosine").await {
        tracing::warn!("Collection create (may already exist): {}", e);
    }
    
    // Store chunks with real embeddings in BarqDB
    let mut insert_errors = 0;
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_id_u64 = hash_to_u64(&chunk.chunk_id);
        let vector = embeddings.get(i).cloned().unwrap_or_else(|| vec![0.0f32; embed_dim]);
        
        let payload = serde_json::json!({
            "document_id": document_id,
            "document_name": document_name,
            "text": chunk.text,
            "sequence_num": chunk.sequence_num,
            "start_position": chunk.start_position,
            "end_position": chunk.end_position,
            "status": "embedded"
        });
        
        if let Err(e) = state_guard.vector_db.insert("sembra_chunks", chunk_id_u64, vector, payload).await {
            tracing::error!("Failed to insert chunk {} into BarqDB: {}", chunk.chunk_id, e);
            insert_errors += 1;
        }
    }
    
    if insert_errors > 0 {
        tracing::error!("{}/{} chunks failed to insert into BarqDB", insert_errors, chunk_count);
    } else {
        info!("Successfully stored {} chunks with embeddings in BarqDB", chunk_count);
    }
    
    // Create graph relationships
    let doc_id_u64 = hash_to_u64(&document_id);
    if let Err(e) = state_guard.graph_db.add_node(doc_id_u64, "Document", serde_json::json!({
        "document_id": document_id,
        "name": document_name,
        "chunk_count": chunk_count
    })).await {
        tracing::error!("Failed to add Document node to GraphDB: {}", e);
    }
    
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_id_u64 = hash_to_u64(&chunk.chunk_id);
        
        if let Err(e) = state_guard.graph_db.add_node(chunk_id_u64, "Chunk", serde_json::json!({
            "chunk_id": chunk.chunk_id,
            "sequence_num": chunk.sequence_num
        })).await {
            tracing::error!("Failed to add Chunk node to GraphDB: {}", e);
        }
        
        if let Err(e) = state_guard.graph_db.add_edge(chunk_id_u64, doc_id_u64, "BELONGS_TO").await {
            tracing::error!("Failed to add BELONGS_TO edge: {}", e);
        }
        
        if i > 0 {
            let prev_chunk_id = hash_to_u64(&chunks[i-1].chunk_id);
            if let Err(e) = state_guard.graph_db.add_edge(prev_chunk_id, chunk_id_u64, "NEXT_CHUNK").await {
                tracing::error!("Failed to add NEXT_CHUNK edge: {}", e);
            }
        }
    }
    
    info!("Stored {} chunks with graph relationships", chunk_count);
    
    // Persist document metadata to Postgres for listing/deletion
    if let Err(e) = state_guard.metadata.insert_document(
        &document_id, &document_name, chunk_count as i32, total_chars as i32
    ).await {
        tracing::error!("Failed to insert document metadata into Postgres: {}", e);
    }
    
    // Store chunk ID mappings for deletion
    for chunk in &chunks {
        let chunk_id_u64 = hash_to_u64(&chunk.chunk_id);
        let graph_node_id = Some(chunk_id_u64);
        if let Err(e) = state_guard.metadata.insert_document_chunk(
            &document_id, chunk_id_u64, graph_node_id
        ).await {
            tracing::error!("Failed to insert chunk mapping: {}", e);
        }
    }
    
    // Also publish to AiMesh queue for any additional background processing
    match sembra_core::aimesh::AiMeshConsumer::from_env().await {
        Ok(consumer) => {
            let messages: Vec<sembra_core::aimesh::ChunkMessage> = chunks.iter().map(|c| {
                sembra_core::aimesh::ChunkMessage {
                    chunk_id: c.chunk_id.clone(),
                    text: c.text.clone(),
                    document_id: c.document_id.clone(),
                    sequence_num: c.sequence_num as u64,
                    metadata: serde_json::json!({
                        "document_name": document_name,
                        "start_position": c.start_position,
                        "end_position": c.end_position
                    }),
                }
            }).collect();
            
            if let Err(e) = consumer.publish_batch(&messages).await {
                tracing::warn!("Failed to publish chunks to AiMesh (non-critical): {}", e);
            } else {
                info!("Published {} chunks to AiMesh queue", messages.len());
            }
        },
        Err(e) => {
            tracing::warn!("AiMesh not available (non-critical): {}", e);
        }
    }
    
    Ok(Json(UploadResponse {
        document_id,
        document_name,
        total_chars,
        chunk_count,
        status: "success".to_string(),
    }))
}
