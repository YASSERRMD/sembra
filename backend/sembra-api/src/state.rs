use std::sync::Arc;
use tokio::sync::RwLock;
use sembra_storage::{MetadataStore, BarqDBClient};
use sembra_graph::BarqGraphDBClient;
use sembra_cache::CelrixCache;
use crate::services::auth::AuthService;
use sembra_core::providers::EmbeddingProvider;

pub struct AppState {
    pub cache: CelrixCache,
    pub metadata: MetadataStore,      // Postgres for Auth/Config
    pub vector_db: BarqDBClient,      // BarqDB for Vectors
    pub graph_db: BarqGraphDBClient,  // BarqGraphDB for Graph
    pub auth: AuthService,
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
}
