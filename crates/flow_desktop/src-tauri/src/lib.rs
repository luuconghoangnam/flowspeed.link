use flow_core::checksum::{ChecksumAlgorithm, ChecksumUtil};
use flow_core::downloader::HttpDownloadCoordinator;
use flow_core::storage::AtomicJsonStorage;
use flow_core::types::AppSettings;
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

fn get_settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("flowspeed")
        .join("settings.json")
}

#[tauri::command]
fn get_settings() -> Result<AppSettings, String> {
    let path = get_settings_path();
    let loaded: Option<AppSettings> = AtomicJsonStorage::load(&path).map_err(|e| e.to_string())?;
    Ok(loaded.unwrap_or_default())
}

#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    AtomicJsonStorage::save_atomic(&path, &settings).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn open_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let clean_path = path.replace('/', "\\");
        let _ = Command::new("explorer")
            .args(["/select,", &clean_path])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let parent = std::path::Path::new(&path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new(&path));
        let _ = Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let clean_path = path.replace('/', "\\");
        let _ = Command::new("explorer")
            .arg(&clean_path)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;
        let _ = Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[tauri::command]
fn calculate_file_checksum(path: String, algorithm: String) -> Result<String, String> {
    let algo = match algorithm.to_lowercase().as_str() {
        "md5" => ChecksumAlgorithm::Md5,
        "sha1" | "sha-1" => ChecksumAlgorithm::Sha1,
        "sha256" | "sha-256" => ChecksumAlgorithm::Sha256,
        _ => return Err("Unsupported checksum algorithm".to_string()),
    };
    ChecksumUtil::calculate_file_checksum(&path, algo).map_err(|e| e.to_string())
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

    // Xác định thư mục lưu mặc định: Downloads của người dùng hoặc cấu hình
    let target_dir = if let Some(folder) = save_folder {
        PathBuf::from(folder)
    } else {
        let settings = get_settings().unwrap_or_default();
        PathBuf::from(settings.download_dir)
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
        eprintln!("[FLOW_DESKTOP] Starting download task {} for URL: {}", task_id_for_finish, t_url);

        let result = coordinator
            .start_download(
                t_id.clone(),
                t_url.clone(),
                HashMap::new(),
                target_dir,
                t_cancel,
                move |event| {
                    let _ = app_handle_for_events.emit("download-progress", &event);
                },
            )
            .await;

        match result {
            Ok(final_path) => {
                eprintln!("[FLOW_DESKTOP COMPLETED] id={}, path={:?}", task_id_for_finish, final_path);
                let _ = t_app.emit(
                    "download-completed",
                    serde_json::json!({
                        "id": task_id_for_finish,
                        "path": final_path.to_string_lossy(),
                    }),
                );
            }
            Err(e) => {
                eprintln!("[FLOW_DESKTOP ERROR] id={}, error={:?}", task_id_for_finish, e);
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
        .invoke_handler(tauri::generate_handler![
            start_download,
            cancel_download,
            get_settings,
            save_settings,
            open_in_folder,
            open_folder,
            calculate_file_checksum
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
