use flow_core::queue::manager::QueueManager;
use flow_server::create_router;
use flow_server::routes::AppState;
use single_instance::SingleInstance;
use std::net::SocketAddr;
use std::sync::Arc;

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
        queue_manager: Arc::new(QueueManager::new()),
    });

    let app = create_router(state);


    let addr = SocketAddr::from(([127, 0, 0, 1], 15151));
    tracing::info!("Flow Speed Link Extension Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
