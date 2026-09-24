use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use slint::{ComponentHandle, ModelRc, VecModel};
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

use flow_core::checksum::{ChecksumAlgorithm, ChecksumUtil};
use flow_core::downloader::coordinator::HttpDownloadCoordinator;
use flow_core::storage::AtomicJsonStorage;
use flow_core::types::AppSettings;

slint::include_modules!();

#[derive(Debug, Clone)]
struct ActiveTaskState {
    id: String,
    url: String,
    file_name: String,
    save_path: PathBuf,
    category: String,
    total_bytes: u64,
    downloaded_bytes: u64,
    speed_bytes_sec: u64,
    threads_count: usize,
    status: String, // "Downloading", "Completed", "Paused", "Failed"
    is_canceled: Arc<AtomicBool>,
}

fn get_settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("flowspeed")
        .join("settings.json")
}

fn load_app_settings() -> AppSettings {
    let path = get_settings_path();
    AtomicJsonStorage::load(&path).ok().flatten().unwrap_or_default()
}

fn save_app_settings_file(settings: &AppSettings) {
    let path = get_settings_path();
    let _ = AtomicJsonStorage::save_atomic(&path, settings);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Flow Speed Link - Pure Native Rust GUI (Slint Engine)...");

    let active_tasks: Arc<Mutex<HashMap<String, ActiveTaskState>>> = Arc::new(Mutex::new(HashMap::new()));
    let speed_history_data: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(vec![0.0; 10]));

    // Create Slint Window
    let main_window = AppWindow::new()?;
    let window_weak = main_window.as_weak();

    // Start background Axum Server on port 15151 for Extension
    let app_state_tasks = active_tasks.clone();
    let w_weak_for_server = window_weak.clone();
    tokio::spawn(async move {
        let app = axum::Router::new().route(
            "/add",
            axum::routing::post({
                let tasks_map = app_state_tasks.clone();
                let w_weak = w_weak_for_server.clone();
                move |body: String| {
                    let tasks_map = tasks_map.clone();
                    async move {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
                            if let Some(url) = parsed.get("url").and_then(|u| u.as_str()) {
                                let url_str = url.to_string();
                                info!("Received URL from Extension: {}", url_str);
                                
                                // Auto start download
                                let task_id = Uuid::new_v4().to_string();
                                let fname = url_str.split('/').last().unwrap_or("download").split('?').next().unwrap_or("download").to_string();
                                let settings = load_app_settings();
                                let target_dir = PathBuf::from(settings.download_dir);
                                let is_cancel = Arc::new(AtomicBool::new(false));

                                let state = ActiveTaskState {
                                    id: task_id.clone(),
                                    url: url_str.clone(),
                                    file_name: fname.clone(),
                                    save_path: target_dir.join(&fname),
                                    category: "general".to_string(),
                                    total_bytes: 0,
                                    downloaded_bytes: 0,
                                    speed_bytes_sec: 0,
                                    threads_count: 8,
                                    status: "Downloading".to_string(),
                                    is_canceled: is_cancel.clone(),
                                };

                                {
                                    let mut map = tasks_map.lock().await;
                                    map.insert(task_id.clone(), state);
                                }

                                let coord = HttpDownloadCoordinator::new(8);
                                let t_id = task_id.clone();
                                let t_url = url_str.clone();
                                let tasks_map_clone = tasks_map.clone();

                                tokio::spawn(async move {
                                    let t_id_for_prog = t_id.clone();
                                    let map_for_prog = tasks_map_clone.clone();
                                    let _ = coord.start_download(
                                        t_id.clone(),
                                        t_url,
                                        HashMap::new(),
                                        target_dir,
                                        is_cancel,
                                        move |event| {
                                            let map = map_for_prog.clone();
                                            let tid = t_id_for_prog.clone();
                                            tokio::spawn(async move {
                                                let mut m = map.lock().await;
                                                if let Some(task) = m.get_mut(&tid) {
                                                    task.downloaded_bytes = event.downloaded_bytes;
                                                    task.total_bytes = event.total_bytes.unwrap_or(0);
                                                    task.speed_bytes_sec = event.speed_bps;
                                                }
                                            });
                                        },
                                    ).await;
                                });
                            }
                        }
                        "{\"status\":\"ok\"}"
                    }
                }
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:15151").await;
        if let Ok(listener) = listener {
            info!("Extension server listening on http://127.0.0.1:15151");
            let _ = axum::serve(listener, app).await;
        }
    });

    // Handle "add_download" callback from Slint UI
    let tasks_add = active_tasks.clone();
    main_window.on_add_download(move |url_str, save_dir_str, threads_count| {
        let url = url_str.to_string();
        let save_dir = save_dir_str.to_string();
        let threads = (threads_count as usize).max(1);
        let tasks_map = tasks_add.clone();

        tokio::spawn(async move {
            let task_id = Uuid::new_v4().to_string();
            let fname = url.split('/').last().unwrap_or("download").split('?').next().unwrap_or("download").to_string();
            let fname = if fname.is_empty() { "download".to_string() } else { fname };
            let target_dir = PathBuf::from(&save_dir);
            let target_file = target_dir.join(&fname);
            let is_cancel = Arc::new(AtomicBool::new(false));

            let state = ActiveTaskState {
                id: task_id.clone(),
                url: url.clone(),
                file_name: fname,
                save_path: target_file,
                category: "general".to_string(),
                total_bytes: 0,
                downloaded_bytes: 0,
                speed_bytes_sec: 0,
                threads_count: threads,
                status: "Downloading".to_string(),
                is_canceled: is_cancel.clone(),
            };

            {
                let mut map = tasks_map.lock().await;
                map.insert(task_id.clone(), state);
            }

            let coord = HttpDownloadCoordinator::new(threads);
            let t_id = task_id.clone();
            let t_url = url.clone();
            let tasks_map_clone = tasks_map.clone();

            tokio::spawn(async move {
                let t_id_for_prog = t_id.clone();
                let map_for_prog = tasks_map_clone.clone();
                let res = coord.start_download(
                    t_id.clone(),
                    t_url,
                    HashMap::new(),
                    target_dir,
                    is_cancel,
                    move |event| {
                        let map = map_for_prog.clone();
                        let tid = t_id_for_prog.clone();
                        tokio::spawn(async move {
                            let mut m = map.lock().await;
                            if let Some(task) = m.get_mut(&tid) {
                                task.downloaded_bytes = event.downloaded_bytes;
                                task.total_bytes = event.total_bytes.unwrap_or(0);
                                task.speed_bytes_sec = event.speed_bps;
                            }
                        });
                    },
                ).await;

                let mut m = tasks_map_clone.lock().await;
                if let Some(task) = m.get_mut(&t_id) {
                    if res.is_ok() {
                        task.status = "Completed".to_string();
                        task.speed_bytes_sec = 0;
                    } else {
                        task.status = "Failed".to_string();
                        task.speed_bytes_sec = 0;
                    }
                }
            });
        });
    });

    // Handle "add_batch_downloads"
    let tasks_batch = active_tasks.clone();
    main_window.on_add_batch_downloads(move |urls_text, save_dir_str, threads_count| {
        let text = urls_text.to_string();
        let save_dir = save_dir_str.to_string();
        let threads = (threads_count as usize).max(1);
        let tasks_map = tasks_batch.clone();

        tokio::spawn(async move {
            for line in text.lines() {
                let url = line.trim().to_string();
                if url.starts_with("http://") || url.starts_with("https://") {
                    let task_id = Uuid::new_v4().to_string();
                    let fname = url.split('/').last().unwrap_or("file").split('?').next().unwrap_or("file").to_string();
                    let target_dir = PathBuf::from(&save_dir);
                    let is_cancel = Arc::new(AtomicBool::new(false));

                    let state = ActiveTaskState {
                        id: task_id.clone(),
                        url: url.clone(),
                        file_name: fname,
                        save_path: target_dir.join(url.split('/').last().unwrap_or("file")),
                        category: "general".to_string(),
                        total_bytes: 0,
                        downloaded_bytes: 0,
                        speed_bytes_sec: 0,
                        threads_count: threads,
                        status: "Downloading".to_string(),
                        is_canceled: is_cancel.clone(),
                    };

                    {
                        let mut map = tasks_map.lock().await;
                        map.insert(task_id.clone(), state);
                    }

                    let coord = HttpDownloadCoordinator::new(threads);
                    let t_id = task_id.clone();
                    let t_url = url.clone();
                    let tasks_map_clone = tasks_map.clone();

                    tokio::spawn(async move {
                        let t_id_for_prog = t_id.clone();
                        let map_for_prog = tasks_map_clone.clone();
                        let res = coord.start_download(
                            t_id.clone(),
                            t_url,
                            HashMap::new(),
                            target_dir,
                            is_cancel,
                            move |event| {
                                let map = map_for_prog.clone();
                                let tid = t_id_for_prog.clone();
                                tokio::spawn(async move {
                                    let mut m = map.lock().await;
                                    if let Some(task) = m.get_mut(&tid) {
                                        task.downloaded_bytes = event.downloaded_bytes;
                                        task.total_bytes = event.total_bytes.unwrap_or(0);
                                        task.speed_bytes_sec = event.speed_bps;
                                    }
                                });
                            },
                        ).await;

                        let mut m = tasks_map_clone.lock().await;
                        if let Some(task) = m.get_mut(&t_id) {
                            if res.is_ok() {
                                task.status = "Completed".to_string();
                            } else {
                                task.status = "Failed".to_string();
                            }
                            task.speed_bytes_sec = 0;
                        }
                    });
                }
            }
        });
    });

    // Handle "pause_task" / "cancel_task"
    let tasks_pause = active_tasks.clone();
    main_window.on_pause_task(move |task_id| {
        let tid = task_id.to_string();
        let tasks_map = tasks_pause.clone();
        tokio::spawn(async move {
            let mut map = tasks_map.lock().await;
            if let Some(task) = map.get_mut(&tid) {
                task.is_canceled.store(true, Ordering::Relaxed);
                task.status = "Paused".to_string();
                task.speed_bytes_sec = 0;
            }
        });
    });

    // Handle "open_folder"
    main_window.on_open_folder(move |path_str| {
        let path = path_str.to_string();
        let _ = open::that(path);
    });

    // Handle Checksum calculation
    main_window.on_calculate_checksum(move |path_str| {
        let path = path_str.to_string();
        tokio::spawn(async move {
            let _ = ChecksumUtil::calculate_file_checksum(&path, ChecksumAlgorithm::Sha256);
        });
    });

    // Timer to update Slint UI model with real metrics
    let timer = slint::Timer::default();
    let tasks_poll = active_tasks.clone();
    let speed_hist_poll = speed_history_data.clone();
    let window_weak_poll = window_weak.clone();

    timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(250), move || {
        let map_arc = tasks_poll.clone();
        let speed_arc = speed_hist_poll.clone();
        let w_weak = window_weak_poll.clone();

        tokio::spawn(async move {
            let map = map_arc.lock().await;
            let mut items = Vec::new();
            let mut total_speed = 0u64;

            for t in map.values() {
                total_speed += t.speed_bytes_sec;

                let size_text = if t.total_bytes > 0 {
                    format!("{:.1} MB / {:.1} MB", t.downloaded_bytes as f64 / 1_048_576.0, t.total_bytes as f64 / 1_048_576.0)
                } else {
                    format!("{:.1} MB", t.downloaded_bytes as f64 / 1_048_576.0)
                };

                let speed_text = if t.speed_bytes_sec > 0 {
                    format!("{:.2} MB/s", t.speed_bytes_sec as f64 / 1_048_576.0)
                } else {
                    "".to_string()
                };

                let progress = if t.total_bytes > 0 {
                    (t.downloaded_bytes as f32 / t.total_bytes as f32).min(1.0)
                } else if t.status == "Completed" {
                    1.0
                } else {
                    0.0
                };

                items.push(TaskItem {
                    id: t.id.clone().into(),
                    url: t.url.clone().into(),
                    file_name: t.file_name.clone().into(),
                    save_path: t.save_path.to_string_lossy().to_string().into(),
                    category: t.category.clone().into(),
                    total_bytes: t.total_bytes as i32,
                    downloaded_bytes: t.downloaded_bytes as i32,
                    progress,
                    speed_bytes_per_sec: t.speed_bytes_sec as i32,
                    speed_text: speed_text.into(),
                    eta_text: "".into(),
                    size_text: size_text.into(),
                    status: t.status.clone().into(),
                    threads_count: t.threads_count as i32,
                    is_hls: false,
                    parts_summary: "".into(),
                });
            }

            let speed_mb_text = format!("{:.2} MB/s", total_speed as f64 / 1_048_576.0);
            let speed_normalized = (total_speed as f32 / 10_000_000.0).min(1.0);

            let mut hist = speed_arc.lock().await;
            hist.remove(0);
            hist.push(speed_normalized);
            let hist_slice = hist.clone();

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = w_weak.upgrade() {
                    let task_model = ModelRc::new(VecModel::from(items));
                    w.set_tasks(task_model);
                    w.set_total_speed_text(speed_mb_text.into());
                    w.set_speed_history(ModelRc::new(VecModel::from(hist_slice)));
                }
            });
        });
    });

    // Run Slint Event Loop
    main_window.run()?;
    Ok(())
}
