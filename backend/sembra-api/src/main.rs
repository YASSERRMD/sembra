use tracing::info;
use std::sync::Arc;
use tokio::sync::RwLock;

use sembra_storage::BarqDB;
use sembra_graph::GraphDB;
use sembra_cache::CelrixCache;
use sembra_api::state::AppState;
use sembra_api::handlers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("sembra_api=info".parse().unwrap()),
        )
        .init();

    info!("Starting SEMBRA API server (Enterprise Edition)...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@postgres:5432/sembra".to_string());
    
    // Initialize connections
    info!("Connecting to BarqDB (Storage & Config) at {}", database_url);
    let barq_db = BarqDB::connect(&database_url, 10).await?;
    let graph_db = GraphDB::connect(&database_url, 10).await?;

    info!("Initializing System Configuration Schema...");
    if let Err(e) = barq_db.init_config_table().await {
        tracing::warn!("Config schema init warning: {}", e);
    }
    
    info!("Initializing Graph Schema...");
    if let Err(e) = graph_db.init_schema().await {
        tracing::warn!("Graph schema init warning: {}", e);
    }

    // Create application state
    let state = Arc::new(RwLock::new(AppState {
        cache: CelrixCache::new(100_000, 86400),
        barq_db,
        graph_db,
    }));

    // Create router using modular handlers
    let app = handlers::create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
