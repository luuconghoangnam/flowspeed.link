use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use flow_core::queue::manager::QueueManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AddDownloadPayload {
    /// Định dạng từ Chrome Extension v1.0.1: { items: [...], options: {...} }
    ExtensionFormat {
        items: Vec<ExtensionDownloadItem>,
        options: Option<ExtensionOptions>,
    },
    /// Định dạng Array tiêu chuẩn từ REST-API.yml: [ { link, ... } ]
    ArrayFormat(Vec<DownloadSourceItem>),
}

#[derive(Debug, Deserialize)]
pub struct ExtensionDownloadItem {
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub link: String,
    pub headers: Option<HashMap<String, String>>,
    #[serde(rename = "downloadPage")]
    pub download_page: Option<String>,
    #[serde(rename = "suggestedName")]
    pub suggested_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExtensionOptions {
    #[serde(rename = "silentAdd")]
    pub silent_add: Option<bool>,
    #[serde(rename = "silentStart")]
    pub silent_start: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadSourceItem {
    pub link: String,
    pub headers: Option<HashMap<String, String>>,
    #[serde(rename = "downloadPage")]
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
    pub download_source: DownloadSourceItem,
    pub folder: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "queueId")]
    pub queue_id: Option<i64>,
}

pub struct AppState {
    pub queue_manager: Arc<QueueManager>,
}

/// Handler cho GET / (Health Check từ Extension)
pub async fn handle_health_check() -> impl IntoResponse {
    (StatusCode::OK, "Flow Speed Link Server Running")
}

/// Handler cho POST /add (Hỗ trợ cả 2 chuẩn payload từ Extension & API)
pub async fn handle_add_downloads(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<AddDownloadPayload>,
) -> impl IntoResponse {
    match payload {
        AddDownloadPayload::ExtensionFormat { items, options } => {
            tracing::info!(
                "Received {} items from Chrome Extension (silentAdd: {:?})",
                items.len(),
                options.as_ref().and_then(|o| o.silent_add)
            );
        }
        AddDownloadPayload::ArrayFormat(items) => {
            tracing::info!("Received {} items from REST API array format", items.len());
        }
    }
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
