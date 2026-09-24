pub mod routes;

use axum::routing::{get, post};
use axum::Router;
use routes::{handle_add_downloads, handle_get_queues, handle_headless_download, handle_health_check, AppState};
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
