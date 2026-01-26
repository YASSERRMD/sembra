use std::sync::Arc;
use tokio::sync::RwLock;
use sembra_storage::BarqDB;
use sembra_graph::GraphDB;
use sembra_cache::CelrixCache;
use crate::services::auth::AuthService;

pub struct AppState {
    pub cache: CelrixCache,
    pub barq_db: BarqDB,
    pub graph_db: GraphDB,
    pub auth: AuthService,
}
