use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use flow_core::queue::manager::QueueManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct AddDownloadRequest {
    pub link: String,
    pub headers: Option<HashMap<String, String>>,
    pub download_page: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QueueItemDto {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct HeadlessDownloadRequest {
    #[serde(rename = "downloadSource")]
    pub download_source: AddDownloadRequest,
    pub folder: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "queueId")]
    pub queue_id: Option<i64>,
}

pub struct AppState {
    pub queue_manager: Arc<QueueManager>,
}

/// Handler cho POST /add
pub async fn handle_add_downloads(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<Vec<AddDownloadRequest>>,
) -> impl IntoResponse {
    tracing::info!("Received {} links to add", payload.len());
    (StatusCode::OK, "OK")
}

/// Handler cho GET /queues
pub async fn handle_get_queues(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let queues = vec![QueueItemDto {
        id: 1,
        name: "Default Queue".to_string(),
    }];
    Json(queues)
}

/// Handler cho POST /start-headless-download
pub async fn handle_headless_download(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<HeadlessDownloadRequest>,
) -> impl IntoResponse {
    tracing::info!("Headless download request: {:?}", payload.name);
    (StatusCode::OK, "OK")
}
