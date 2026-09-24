use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use flow_core::queue::manager::QueueManager;
use flow_core::types::{DownloadStatus, DownloadTask, DownloadType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddDownloadPayload>,
) -> impl IntoResponse {
    let mut added_count = 0;

    match payload {
        AddDownloadPayload::ExtensionFormat { items, options: _ } => {
            for item in items {
                let filename = item
                    .suggested_name
                    .unwrap_or_else(|| extract_filename(&item.link));

                let is_m3u8 = item.link.contains(".m3u8")
                    || item.item_type.as_deref() == Some("hls");

                let task = DownloadTask {
                    id: Uuid::new_v4().to_string(),
                    url: item.link,
                    filename,
                    save_path: "".to_string(),
                    total_bytes: None,
                    downloaded_bytes: 0,
                    status: DownloadStatus::Queued,
                    download_type: if is_m3u8 {
                        DownloadType::HlsStream
                    } else {
                        DownloadType::HttpRange
                    },
                    parts: vec![],
                    headers: item.headers.unwrap_or_default(),
                    queue_id: Some(1),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                state.queue_manager.add_task(task).await;
                added_count += 1;
            }
        }
        AddDownloadPayload::ArrayFormat(items) => {
            for item in items {
                let filename = extract_filename(&item.link);
                let is_m3u8 = item.link.contains(".m3u8");

                let task = DownloadTask {
                    id: Uuid::new_v4().to_string(),
                    url: item.link,
                    filename,
                    save_path: "".to_string(),
                    total_bytes: None,
                    downloaded_bytes: 0,
                    status: DownloadStatus::Queued,
                    download_type: if is_m3u8 {
                        DownloadType::HlsStream
                    } else {
                        DownloadType::HttpRange
                    },
                    parts: vec![],
                    headers: item.headers.unwrap_or_default(),
                    queue_id: Some(1),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                state.queue_manager.add_task(task).await;
                added_count += 1;
            }
        }
    }

    (StatusCode::OK, format!("Added {} tasks", added_count))
}

/// Handler cho GET /queues
pub async fn handle_get_queues(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let queues = state.queue_manager.get_all_queues().await;
    let dtos: Vec<QueueItemDto> = queues
        .into_iter()
        .map(|q| QueueItemDto {
            id: q.id,
            name: q.name,
        })
        .collect();
    Json(dtos)
}

/// Handler cho POST /start-headless-download
pub async fn handle_headless_download(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<HeadlessDownloadRequest>,
) -> impl IntoResponse {
    let filename = payload
        .name
        .unwrap_or_else(|| extract_filename(&payload.download_source.link));

    let task = DownloadTask {
        id: Uuid::new_v4().to_string(),
        url: payload.download_source.link,
        filename,
        save_path: payload.folder.unwrap_or_default(),
        total_bytes: None,
        downloaded_bytes: 0,
        status: DownloadStatus::Downloading,
        download_type: DownloadType::HttpRange,
        parts: vec![],
        headers: payload.download_source.headers.unwrap_or_default(),
        queue_id: payload.queue_id.or(Some(1)),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    state.queue_manager.add_task(task).await;
    (StatusCode::OK, "OK")
}

fn extract_filename(url: &str) -> String {
    url.split('/')
        .last()
        .map(|s| s.split('?').next().unwrap_or(s))
        .filter(|s| !s.is_empty())
        .unwrap_or("download")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::{get, post};
    use axum::Router;
    use http_body_util::BodyExt;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_rest_api_health_check_and_queues() {
        let state = Arc::new(AppState {
            queue_manager: Arc::new(QueueManager::new()),
        });

        let app = Router::new()
            .route("/", get(handle_health_check))
            .route("/add", post(handle_add_downloads))
            .route("/queues", get(handle_get_queues))
            .with_state(state.clone());

        // Test GET /
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test POST /add với Chrome Extension format
        let payload = serde_json::json!({
            "items": [
                {
                    "link": "https://example.com/video.mp4",
                    "suggestedName": "video.mp4"
                }
            ],
            "options": {
                "silentAdd": true
            }
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/add")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Kiểm tra task đã được thêm vào QueueManager
        let tasks = state.queue_manager.get_all_tasks().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].filename, "video.mp4");

        // Test GET /queues
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/queues")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let queues: Vec<QueueItemDto> = serde_json::from_slice(&body).unwrap();
        assert_eq!(queues.len(), 1);
        assert_eq!(queues[0].name, "Mặc định");
    }
}

