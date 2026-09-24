use flow_core::checksum::{ChecksumAlgorithm, ChecksumUtil};
use flow_core::downloader::HttpDownloadCoordinator;
use flow_core::queue::manager::QueueManager;
use flow_core::storage::AtomicJsonStorage;
use flow_core::types::AppSettings;
use flow_server::routes::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;
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
    pub queue_manager: Arc<QueueManager>,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            active_cancellations: Arc::new(Mutex::new(HashMap::new())),
            queue_manager: Arc::new(QueueManager::new()),
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

    let settings = get_settings().unwrap_or_default();

    // Xác định tên file ước lượng
    let filename = url
        .split('/')
        .last()
        .map(|s| s.split('?').next().unwrap_or(s))
        .unwrap_or("download")
        .to_string();

    // Tự động phân loại danh mục thư mục nếu chưa chỉ định folder
    let target_dir = if let Some(folder) = save_folder {
        PathBuf::from(folder)
    } else {
        let mut matched_folder = None;
        for cat in &settings.categories {
            if cat.matches_filename(&filename) {
                if let Some(ref cf) = cat.custom_folder {
                    if !cf.trim().is_empty() {
                        matched_folder = Some(PathBuf::from(cf));
                        break;
                    }
                }
            }
        }
        matched_folder.unwrap_or_else(|| PathBuf::from(settings.download_dir))
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
    let notify_enabled = settings.notification_enabled;

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
                let path_str = final_path.to_string_lossy().to_string();
                let fname = final_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Tệp tin".to_string());
                eprintln!("[FLOW_DESKTOP COMPLETED] id={}, path={}", task_id_for_finish, path_str);
                
                let _ = t_app.emit(
                    "download-completed",
                    serde_json::json!({
                        "id": task_id_for_finish,
                        "path": path_str,
                        "filename": fname,
                    }),
                );

                if notify_enabled {
                    let _ = t_app.notification()
                        .builder()
                        .title("Flow Speed Link — Tải hoàn tất! 🎉")
                        .body(format!("Đã tải thành công: {}", fname))
                        .show();
                }
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
    let desktop_state = DesktopState::default();
    let queue_mgr = desktop_state.queue_manager.clone();

    tauri::Builder::default()
        .manage(desktop_state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // 1. Khởi chạy ngầm Axum REST API cho Chrome/Firefox Extension (port 15151)
            let server_state = Arc::new(AppState {
                queue_manager: queue_mgr.clone(),
            });
            let addr = SocketAddr::from(([127, 0, 0, 1], 15151));

            tauri::async_runtime::spawn(async move {
                eprintln!("[FLOW_SERVER] Embedded Extension Server starting on http://{}", addr);
                if let Err(e) = flow_server::run_server(addr, server_state).await {
                    eprintln!("[FLOW_SERVER] Embedded server error: {:?}", e);
                }
            });

            // 2. Tạo Menu System Tray
            let show_i = MenuItem::with_id(app, "show", "Mở Flow Speed Link", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "Ẩn xuống khay", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Thoát ứng dụng", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Flow Speed Link — Trình Tải Siêu Tốc")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
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

