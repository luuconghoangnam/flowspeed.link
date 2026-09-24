use crate::downloader::part::{DownloadError, HttpPartDownloader};
use crate::storage::sparse::mark_as_sparse_file;
use crate::types::{DownloadProgressEvent, DownloadStatus, DownloadTask, PartInfo};
use chrono::Utc;
use reqwest::header::{ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH};
use reqwest::Client;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Thông tin thăm dò trước khi tải
#[derive(Debug, Clone)]
pub struct ProbeInfo {
    pub total_bytes: Option<u64>,
    pub supports_range: bool,
    pub suggested_filename: Option<String>,
}

pub struct HttpDownloadCoordinator {
    client: Client,
}

impl HttpDownloadCoordinator {
    pub fn new() -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(16)
            .tcp_nodelay(true)
            .build()
            .unwrap_or_default();

        Self { client }
    }

    /// Thăm dò thông tin file từ URL (HEAD request hoặc GET range 0-0)
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.download.HttpDownloadJob.prepare()
    pub async fn probe(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<ProbeInfo, DownloadError> {
        let mut req = self.client.head(url);
        for (k, v) in headers {
            req = req.header(k, v);
        }

        let resp = match req.send().await {
            Ok(r) if r.status().is_success() => r,
            _ => {
                // Fallback: Thử GET dải 0-0 nếu HEAD bị server từ chối
                let mut get_req = self.client.get(url).header("Range", "bytes=0-0");
                for (k, v) in headers {
                    get_req = get_req.header(k, v);
                }
                get_req.send().await?
            }
        };

        let headers_map = resp.headers();

        // 1. Kiểm tra Content-Length
        let total_bytes = headers_map
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        // 2. Kiểm tra hỗ trợ dải Range (Accept-Ranges: bytes hoặc status 206)
        let supports_range = headers_map
            .get(ACCEPT_RANGES)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("bytes"))
            .unwrap_or(false)
            || resp.status().as_u16() == 206;

        // 3. Trích xuất tên file từ Content-Disposition
        let suggested_filename = headers_map
            .get(CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_filename_from_disposition);

        Ok(ProbeInfo {
            total_bytes,
            supports_range,
            suggested_filename,
        })
    }

    /// Khởi tạo và chia các Part (dải byte) cho tác vụ tải
    pub fn create_parts(total_bytes: u64, num_parts: usize) -> Vec<PartInfo> {
        if num_parts <= 1 || total_bytes < num_parts as u64 {
            return vec![PartInfo {
                index: 0,
                start_byte: 0,
                end_byte: total_bytes.saturating_sub(1),
                downloaded_bytes: 0,
                is_completed: false,
            }];
        }

        let part_size = total_bytes / num_parts as u64;
        let mut parts = Vec::with_capacity(num_parts);

        for i in 0..num_parts {
            let start_byte = i as u64 * part_size;
            let end_byte = if i == num_parts - 1 {
                total_bytes - 1
            } else {
                start_byte + part_size - 1
            };

            parts.push(PartInfo {
                index: i,
                start_byte,
                end_byte,
                downloaded_bytes: 0,
                is_completed: false,
            });
        }

        parts
    }

    /// Bắt đầu điều phối tiến trình tải đa luồng
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.download.HttpDownloadJob.start()
    pub async fn execute_download(
        &self,
        task: &mut DownloadTask,
        num_threads: usize,
        progress_tx: Option<mpsc::Sender<DownloadProgressEvent>>,
        is_paused: Arc<AtomicBool>,
        is_canceled: Arc<AtomicBool>,
    ) -> Result<(), DownloadError> {
        let save_path = PathBuf::from(&task.save_path);
        if let Some(parent) = save_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // 1. Thăm dò thông tin nếu chưa có total_bytes
        if task.total_bytes.is_none() {
            let probe = self.probe(&task.url, &task.headers).await?;
            task.total_bytes = probe.total_bytes;
        }

        // 2. Cấp phát Sparse file nếu có dung lượng xác định
        if let Some(total) = task.total_bytes {
            let file = File::create(&save_path)?;
            file.set_len(total)?;
            let _ = mark_as_sparse_file(&file);

            if task.parts.is_empty() {
                task.parts = Self::create_parts(total, num_threads);
            }
        } else if task.parts.is_empty() {
            task.parts = vec![PartInfo {
                index: 0,
                start_byte: 0,
                end_byte: u64::MAX,
                downloaded_bytes: 0,
                is_completed: false,
            }];
        }

        let downloaded_counter = Arc::new(AtomicU64::new(task.downloaded_bytes));

        // 3. Tiến trình theo dõi tốc độ và phát Event (Speed & Progress Meter)
        let meter_id = task.id.clone();
        let meter_total = task.total_bytes;
        let meter_counter = Arc::clone(&downloaded_counter);
        let meter_paused = Arc::clone(&is_paused);
        let meter_canceled = Arc::clone(&is_canceled);
        let meter_tx = progress_tx.clone();

        let meter_handle = tokio::spawn(async move {
            let mut last_bytes = meter_counter.load(Ordering::Relaxed);
            let mut last_time = Instant::now();

            loop {
                sleep(Duration::from_millis(500)).await;

                if meter_canceled.load(Ordering::Relaxed) || meter_paused.load(Ordering::Relaxed) {
                    break;
                }

                let current_bytes = meter_counter.load(Ordering::Relaxed);
                let elapsed_secs = last_time.elapsed().as_secs_f64();

                if elapsed_secs > 0.0 {
                    let bytes_diff = current_bytes.saturating_sub(last_bytes);
                    let speed_bps = (bytes_diff as f64 / elapsed_secs) as u64;

                    let eta_seconds = if speed_bps > 0 && meter_total.is_some() {
                        let remaining = meter_total.unwrap().saturating_sub(current_bytes);
                        Some(remaining / speed_bps)
                    } else {
                        None
                    };

                    if let Some(tx) = &meter_tx {
                        let _ = tx
                            .send(DownloadProgressEvent {
                                id: meter_id.clone(),
                                downloaded_bytes: current_bytes,
                                total_bytes: meter_total,
                                speed_bps,
                                eta_seconds,
                                status: DownloadStatus::Downloading,
                            })
                            .await;
                    }

                    last_bytes = current_bytes;
                    last_time = Instant::now();
                }
            }
        });

        // 4. Spawn các worker tải từng Part song song
        let mut handles = Vec::new();
        for part in &mut task.parts {
            if part.is_completed {
                continue;
            }

            let mut part_clone = part.clone();
            let url = task.url.clone();
            let headers = task.headers.clone();
            let path = save_path.clone();
            let counter = Arc::clone(&downloaded_counter);
            let canceled = Arc::clone(&is_canceled);
            let client = self.client.clone();

            let handle = tokio::spawn(async move {
                let downloader = HttpPartDownloader { client };
                downloader
                    .download_part(
                        &url,
                        &headers,
                        &mut part_clone,
                        path,
                        counter,
                        canceled,
                    )
                    .await
                    .map(|_| part_clone)
            });

            handles.push(handle);
        }

        // 5. Chờ toàn bộ các luồng tải xong
        let mut has_error = None;
        for handle in handles {
            match handle.await {
                Ok(Ok(updated_part)) => {
                    if let Some(p) = task.parts.get_mut(updated_part.index) {
                        *p = updated_part;
                    }
                }
                Ok(Err(e)) => has_error = Some(e),
                Err(_) => has_error = Some(DownloadError::Canceled),
            }
        }

        let _ = meter_handle.await;
        task.downloaded_bytes = downloaded_counter.load(Ordering::Relaxed);
        task.updated_at = Utc::now();

        if let Some(err) = has_error {
            task.status = DownloadStatus::Error(err.to_string());
            return Err(err);
        }

        if is_canceled.load(Ordering::Relaxed) {
            task.status = DownloadStatus::Paused;
            return Err(DownloadError::Canceled);
        }

        task.status = DownloadStatus::Completed;
        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(DownloadProgressEvent {
                    id: task.id.clone(),
                    downloaded_bytes: task.downloaded_bytes,
                    total_bytes: task.total_bytes,
                    speed_bps: 0,
                    eta_seconds: Some(0),
                    status: DownloadStatus::Completed,
                })
                .await;
        }

        Ok(())
    }
}

fn parse_filename_from_disposition(header_val: &str) -> Option<String> {
    for part in header_val.split(';') {
        let part = part.trim();
        if part.starts_with("filename*=") {
            if let Some(name) = part.strip_prefix("filename*=") {
                let clean = name.trim_matches('"').trim();
                if let Some(idx) = clean.find("''") {
                    return Some(clean[idx + 2..].to_string());
                }
                return Some(clean.to_string());
            }
        } else if part.starts_with("filename=") {
            if let Some(name) = part.strip_prefix("filename=") {
                return Some(name.trim_matches('"').trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_parts_slicing() {
        let total_bytes = 1000;
        let parts = HttpDownloadCoordinator::create_parts(total_bytes, 4);

        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].start_byte, 0);
        assert_eq!(parts[0].end_byte, 249);
        assert_eq!(parts[1].start_byte, 250);
        assert_eq!(parts[1].end_byte, 499);
        assert_eq!(parts[2].start_byte, 500);
        assert_eq!(parts[2].end_byte, 749);
        assert_eq!(parts[3].start_byte, 750);
        assert_eq!(parts[3].end_byte, 999);
    }

    #[test]
    fn test_parse_filename_from_content_disposition() {
        let val = r#"attachment; filename="ubuntu-24.04.iso""#;
        assert_eq!(
            parse_filename_from_disposition(val),
            Some("ubuntu-24.04.iso".to_string())
        );

        let val_utf8 = "attachment; filename*=UTF-8''my%20archive.zip";
        assert_eq!(
            parse_filename_from_disposition(val_utf8),
            Some("my%20archive.zip".to_string())
        );
    }
}
