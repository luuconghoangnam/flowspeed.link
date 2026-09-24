use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trạng thái của một tiến trình tải file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Connecting,
    Downloading,
    Paused,
    Completed,
    Error(String),
}

/// Loại tác vụ tải
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadType {
    HttpRange,
    HlsStream,
}

/// Thông tin về từng Part (luồng tải con)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartInfo {
    pub index: usize,
    pub start_byte: u64,
    pub end_byte: u64,
    pub downloaded_bytes: u64,
    pub is_completed: bool,
}

/// Một nhiệm vụ tải (Download Task) hoàn chỉnh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub save_path: String,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
    pub download_type: DownloadType,
    pub parts: Vec<PartInfo>,
    pub headers: HashMap<String, String>,
    pub queue_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cập nhật tiến độ tải gửi lên UI / IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressEvent {
    pub id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bps: u64,
    pub eta_seconds: Option<u64>,
    pub status: DownloadStatus,
}

/// Hàng đợi tải file (Download Queue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadQueue {
    pub id: i64,
    pub name: String,
    pub max_concurrent: usize,
    pub speed_limit_kbps: Option<u64>,
    pub auto_start_time: Option<String>,
    pub auto_stop_time: Option<String>,
    pub is_active: bool,
}

impl Default for DownloadQueue {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Mặc định".to_string(),
            max_concurrent: 3,
            speed_limit_kbps: None,
            auto_start_time: None,
            auto_stop_time: None,
            is_active: true,
        }
    }
}

/// Danh mục tệp tin tự động phân loại (File Category)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub custom_folder: Option<String>,
    pub file_extensions: Vec<String>,
    pub url_patterns: Vec<String>,
}

impl FileCategory {
    pub fn default_categories() -> Vec<Self> {
        vec![
            FileCategory {
                id: "video".to_string(),
                name: "Video & Phim ảnh".to_string(),
                icon: "video".to_string(),
                custom_folder: None,
                file_extensions: vec![
                    "mp4".into(), "mkv".into(), "avi".into(), "mov".into(),
                    "webm".into(), "ts".into(), "flv".into(), "m3u8".into(),
                ],
                url_patterns: vec![],
            },
            FileCategory {
                id: "audio".to_string(),
                name: "Âm nhạc & Audio".to_string(),
                icon: "music".to_string(),
                custom_folder: None,
                file_extensions: vec![
                    "mp3".into(), "flac".into(), "wav".into(), "aac".into(),
                    "ogg".into(), "m4a".into(), "wma".into(),
                ],
                url_patterns: vec![],
            },
            FileCategory {
                id: "compressed".to_string(),
                name: "Tập tin nén & ISO".to_string(),
                icon: "archive".to_string(),
                custom_folder: None,
                file_extensions: vec![
                    "zip".into(), "rar".into(), "7z".into(), "tar".into(),
                    "gz".into(), "bz2".into(), "xz".into(), "iso".into(),
                ],
                url_patterns: vec![],
            },
            FileCategory {
                id: "documents".to_string(),
                name: "Tài liệu văn phòng".to_string(),
                icon: "document".to_string(),
                custom_folder: None,
                file_extensions: vec![
                    "pdf".into(), "docx".into(), "doc".into(), "xlsx".into(),
                    "xls".into(), "pptx".into(), "txt".into(), "epub".into(),
                ],
                url_patterns: vec![],
            },
            FileCategory {
                id: "programs".to_string(),
                name: "Phần mềm & Ứng dụng".to_string(),
                icon: "cpu".to_string(),
                custom_folder: None,
                file_extensions: vec![
                    "exe".into(), "msi".into(), "apk".into(), "dmg".into(),
                    "pkg".into(), "deb".into(), "rpm".into(),
                ],
                url_patterns: vec![],
            },
        ]
    }

    pub fn matches_filename(&self, filename: &str) -> bool {
        let lower = filename.to_lowercase();
        self.file_extensions.iter().any(|ext| {
            let dot_ext = format!(".{}", ext.to_lowercase());
            lower.ends_with(&dot_ext)
        })
    }
}

/// Quy tắc cấu hình kết nối theo từng Tên miền / Host
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerHostRule {
    pub id: String,
    pub domain_pattern: String,
    pub max_threads: Option<usize>,
    pub speed_limit_kbps: Option<u64>,
    pub custom_user_agent: Option<String>,
    pub custom_headers: HashMap<String, String>,
}

/// Hành động nguồn khi hoàn tất tác vụ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PowerActionType {
    Shutdown,
    Sleep,
    Hibernate,
    ExitApp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PowerActionConfig {
    pub action_type: PowerActionType,
    pub force: bool,
    pub countdown_seconds: u32,
    pub is_active: bool,
}

/// Cấu hình toàn cục của ứng dụng (Settings & Preferences)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_dir: String,
    pub default_threads: usize,
    pub max_concurrent_downloads: usize,
    pub auto_start: bool,
    pub server_port: u16,
    pub speed_limit_kbps: Option<u64>,
    pub notification_enabled: bool,
    pub categories: Vec<FileCategory>,
    pub per_host_rules: Vec<PerHostRule>,
    pub power_action: Option<PowerActionConfig>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        Self {
            download_dir,
            default_threads: 8,
            max_concurrent_downloads: 3,
            auto_start: false,
            server_port: 15151,
            speed_limit_kbps: None,
            notification_enabled: true,
            categories: FileCategory::default_categories(),
            per_host_rules: vec![],
            power_action: None,
        }
    }
}


