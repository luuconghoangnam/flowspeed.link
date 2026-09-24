pub mod routes;

use axum::routing::{get, post};
use axum::Router;
use routes::{handle_add_downloads, handle_get_queues, handle_headless_download, handle_health_check, AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(handle_health_check))
        .route("/add", post(handle_add_downloads))
        .route("/queues", get(handle_get_queues))
        .route("/start-headless-download", post(handle_headless_download))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn run_server(addr: SocketAddr, state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
