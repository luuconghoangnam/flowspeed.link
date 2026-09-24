use flow_core::downloader::HttpDownloadCoordinator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTaskDto {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub status: String,
}

pub struct DesktopState {
    pub active_cancellations: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            active_cancellations: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tauri::command]
async fn start_download(
    app: AppHandle,
    state: State<'_, DesktopState>,
    url: String,
    save_folder: Option<String>,
    threads: Option<usize>,
) -> Result<DownloadTaskDto, String> {
    let task_id = Uuid::new_v4().to_string();
    let thread_count = threads.unwrap_or(8);

    // Xác định thư mục lưu mặc định: Downloads của người dùng
    let target_dir = if let Some(folder) = save_folder {
        PathBuf::from(folder)
    } else {
        dirs::download_dir().unwrap_or_else(|| PathBuf::from("."))
    };

    let is_canceled = Arc::new(AtomicBool::new(false));
    {
        let mut map = state.active_cancellations.lock().await;
        map.insert(task_id.clone(), is_canceled.clone());
    }

    let coordinator = HttpDownloadCoordinator::new(thread_count);

    let t_id = task_id.clone();
    let t_url = url.clone();
    let t_app = app.clone();
    let t_cancel = is_canceled.clone();

    // Spawn download runner trên tokio async runtime
    tokio::spawn(async move {
        let app_handle_for_events = t_app.clone();
        let task_id_for_finish = t_id.clone();

        let result = coordinator
            .start_download(
                t_id.clone(),
                t_url.clone(),
                HashMap::new(),
                target_dir,
                t_cancel,
                move |event| {
                    let _ = app_handle_for_events.emit("download-progress", event);
                },
            )
            .await;

        match result {
            Ok(final_path) => {
                let _ = t_app.emit(
                    "download-completed",
                    serde_json::json!({
                        "id": task_id_for_finish,
                        "path": final_path.to_string_lossy(),
                    }),
                );
            }
            Err(e) => {
                let _ = t_app.emit(
                    "download-error",
                    serde_json::json!({
                        "id": task_id_for_finish,
                        "error": e.to_string(),
                    }),
                );
            }
        }
    });

    let filename = url
        .split('/')
        .last()
        .map(|s| s.split('?').next().unwrap_or(s))
        .unwrap_or("download")
        .to_string();

    Ok(DownloadTaskDto {
        id: task_id,
        url,
        filename,
        status: "downloading".to_string(),
    })
}

#[tauri::command]
async fn cancel_download(
    state: State<'_, DesktopState>,
    task_id: String,
) -> Result<(), String> {
    let mut map = state.active_cancellations.lock().await;
    if let Some(token) = map.remove(&task_id) {
        token.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DesktopState::default())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .invoke_handler(tauri::generate_handler![start_download, cancel_download])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
