//! Intelligent chunking with configurable overlap

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A text chunk with metadata
#[derive(Debug, Clone)]
pub struct Chunk {
    pub chunk_id: String,
    pub document_id: String,
    pub text: String,
    pub start_position: usize,
    pub end_position: usize,
    pub sequence_num: usize,
}

/// Generate a deterministic chunk ID
fn generate_chunk_id(document_id: &str, sequence: usize) -> String {
    format!("{}:chunk:{}", document_id, sequence)
}

/// Split text into chunks with overlap
/// 
/// # Arguments
/// * `text` - The full text to chunk
/// * `document_id` - Document identifier
/// * `chunk_size` - Target size of each chunk in characters
/// * `overlap` - Number of characters to overlap between chunks
/// 
/// # Returns
/// Vector of Chunk structs with metadata
pub fn chunk_with_overlap(
    text: &str,
    document_id: &str,
    chunk_size: usize,
    overlap: usize,
) -> Vec<Chunk> {
    let chars: Vec<char> = text.chars().collect();
    let total_len = chars.len();
    
    if total_len == 0 {
        return vec![];
    }
    
    // Ensure overlap is smaller than chunk size
    let effective_overlap = overlap.min(chunk_size.saturating_sub(1));
    let step = chunk_size.saturating_sub(effective_overlap).max(1);
    
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut sequence = 0;
    
    while start < total_len {
        let end = (start + chunk_size).min(total_len);
        let chunk_text: String = chars[start..end].iter().collect();
        
        chunks.push(Chunk {
            chunk_id: generate_chunk_id(document_id, sequence),
            document_id: document_id.to_string(),
            text: chunk_text,
            start_position: start,
            end_position: end,
            sequence_num: sequence,
        });
        
        sequence += 1;
        start += step;
        
        // Break if we've covered all text
        if end >= total_len {
            break;
        }
    }
    
    chunks
}

/// Parallel chunking for large documents
pub async fn chunk_parallel(
    text: &str,
    document_id: &str,
    chunk_size: usize,
    overlap: usize,
) -> Vec<Chunk> {
    // For now, use single-threaded chunking
    // Tokio spawn_blocking for CPU-bound work
    let text = text.to_string();
    let doc_id = document_id.to_string();
    
    tokio::task::spawn_blocking(move || {
        chunk_with_overlap(&text, &doc_id, chunk_size, overlap)
    })
    .await
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_chunking() {
        let text = "Hello world this is a test of chunking functionality";
        let chunks = chunk_with_overlap(text, "doc:1", 10, 2);
        
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].sequence_num, 0);
        assert_eq!(chunks[0].document_id, "doc:1");
    }
    
    #[test]
    fn test_overlap() {
        let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let chunks = chunk_with_overlap(text, "doc:1", 10, 3);
        
        // First chunk: ABCDEFGHIJ (0-10)
        // Second chunk: HIJKLMNOPQ (7-17) - overlaps by 3
        assert!(chunks.len() >= 3);
        
        // Verify overlap exists
        let c1_end = &chunks[0].text[chunks[0].text.len()-3..];
        let c2_start = &chunks[1].text[..3];
        assert_eq!(c1_end, c2_start);
    }
    
    #[test]
    fn test_empty_text() {
        let chunks = chunk_with_overlap("", "doc:1", 10, 2);
        assert!(chunks.is_empty());
    }
}
