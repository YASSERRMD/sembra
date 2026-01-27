use std::env;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn};
use anyhow::Result;

use sembra_core::aimesh::{AiMeshConsumer, ChunkMessage};
use sembra_core::providers::{self, EmbeddingProvider};
use sembra_storage::{MetadataStore, BarqDBClient};
use sembra_cache::CelrixCache;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("embedding_worker=info".parse().unwrap()),
        )
        .init();

    info!("Starting Sembra Embedding Worker...");

    // 1. Initialize Infrastructure Clients
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/sembra".to_string());
    
    info!("Connecting to Metadata Store...");
    let metadata = MetadataStore::connect(&database_url, 10).await?;

    info!("Connecting to BarqDB...");
    let vector_db = BarqDBClient::from_env()?;

    info!("Connecting to Celrix Cache...");
    let cache = CelrixCache::from_env();

    info!("Connecting to AiMesh...");
    let consumer = AiMeshConsumer::from_env().await?;

    info!("All infrastructure connected.");

    // 2. Load Configuration & Initialize Provider
    // In a real production worker, we might reload this periodically or listen for config changes.
    // For Phase 1, we load on startup.
    let provider_type = metadata.get_config("embedding_provider").await.ok().flatten().unwrap_or("mock".into());
    let model = metadata.get_config("embedding_model").await.ok().flatten().unwrap_or("default".into());
    
    info!("Initializing Provider: {} (model: {})", provider_type, model);

    let provider: Arc<dyn EmbeddingProvider> = match provider_type.as_str() {
        "openai" => {
            let api_key = metadata.get_config("openai_api_key").await.ok().flatten()
                .unwrap_or_else(|| env::var("OPENAI_API_KEY").unwrap_or_default());
            Arc::new(providers::openai::OpenAIProvider::new(api_key, model))
        },
        "ollama" => {
            let base_url = env::var("OLLAMA_BASE_URL").unwrap_or("http://localhost:11434".into());
            Arc::new(providers::ollama::OllamaProvider::new(base_url, model))
        },
        _ => {
            warn!("Using MOCK provider. Embeddings will be random noise.");
            Arc::new(providers::mock::MockProvider::new())
        }
    };

    // 3. Processing Loop
    let batch_size = 50;
    info!("Worker ready. Listening for chunks...");

    loop {
        match consumer.read_batch(batch_size).await {
            Ok(messages) => {
                if messages.is_empty() {
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }

                info!("Received batch of {} chunks", messages.len());
                let _batch_id = uuid::Uuid::new_v4().to_string(); // In real AiMesh, this comes from response

                if let Err(e) = process_batch(&messages, &provider, &vector_db, &cache).await {
                    error!("Failed to process batch: {}", e);
                    // In a real system, we might NACK or DLQ here.
                    // For now, we just sleep and retry (or drop if fatal).
                    sleep(Duration::from_secs(5)).await;
                } else {
                    // Commit logic would go here if AiMesh requires explicit commit
                    // consumer.commit_batch(&batch_id).await?;
                    info!("Batch processed successfully.");
                }
            }
            Err(e) => {
                error!("Error reading from AiMesh: {}", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn process_batch(
    messages: &[ChunkMessage],
    provider: &Arc<dyn EmbeddingProvider>,
    vector_db: &BarqDBClient,
    _cache: &CelrixCache,
) -> Result<()> {
    // 1. Prepare texts
    let texts: Vec<String> = messages.iter().map(|m| m.text.clone()).collect();
    
    // 2. Generate Embeddings (with cache check optimization TODO)
    // For Phase 1, we just re-embed to ensure freshness/simplicity.
    // In Phase 2, we should check cache.get(hash(text)) first.
    
    let embeddings = provider.embed_documents(&texts).await?;
    
    if embeddings.len() != messages.len() {
        return Err(anyhow::anyhow!("Provider returned {} embeddings for {} chunks", embeddings.len(), messages.len()));
    }

    // 3. Store in BarqDB
    // We update the existing placeholder records or insert new ones.
    // Since we created records in `upload_handler` with "pending_embedding",
    // we technically should iterate and update.
    // BarqDB `insert` is typically an upsert if ID matches.
    
    for (i, msg) in messages.iter().enumerate() {
        let embedding = &embeddings[i];
        
        // We need to map string ID to u64 for BarqDB
        let chunk_id_u64 = hash_to_u64(&msg.chunk_id);
        
        let payload = serde_json::json!({
            "chunk_id": msg.chunk_id,
            "document_id": msg.document_id,
            "text": msg.text,
            "sequence_num": msg.sequence_num,
            "metadata": msg.metadata,
            "status": "indexed",
            "indexed_at": chrono::Utc::now().to_rfc3339()
        });

        vector_db.insert(
            "sembra_chunks",
            chunk_id_u64,
            embedding.clone(),
            payload
        ).await?;
        
        // 4. Update Cache (L1)
        // Store the embedding in Celrix for fast retrieval if we see this text again
        // cache.set(hash(text), embedding) - TODO in Phase 2 optimization
    }

    Ok(())
}

fn hash_to_u64(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
