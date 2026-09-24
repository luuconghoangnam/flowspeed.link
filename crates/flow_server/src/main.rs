mod routes;

use axum::routing::{get, post};
use axum::Router;
use flow_core::queue::manager::QueueManager;
use routes::{handle_add_downloads, handle_get_queues, handle_headless_download, AppState};
use single_instance::SingleInstance;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Khóa đơn phiên (Single instance lock)
    let instance = SingleInstance::new("flowspeed_link_app_lock")?;
    if !instance.is_single() {
        tracing::warn!("Another instance of Flow Speed Link is already running. Exiting...");
        return Ok(());
    }

    let state = Arc::new(AppState {
        queue_manager: Arc::new(QueueManager::new(3)),
    });

    let app = Router::new()
        .route("/add", post(handle_add_downloads))
        .route("/queues", get(handle_get_queues))
        .route("/start-headless-download", post(handle_headless_download))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 15151));
    tracing::info!("Flow Speed Link Extension Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
