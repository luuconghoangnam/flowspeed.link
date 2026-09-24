use crate::downloader::part::{DownloadError, HttpPartDownloader};
use crate::downloader::probe::UrlProber;
use crate::downloader::speed::SpeedMeter;
use crate::storage::sparse::mark_as_sparse_file;
use crate::types::{DownloadProgressEvent, DownloadStatus, PartInfo};
use reqwest::Client;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

pub struct HttpDownloadCoordinator {
    pub client: Client,
    pub thread_count: usize,
}

impl HttpDownloadCoordinator {
    pub fn new(thread_count: usize) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(20)
            .tcp_nodelay(true)
            .build()
            .unwrap_or_default();

        Self {
            client,
            thread_count: thread_count.max(1),
        }
    }

    /// Khởi chạy toàn bộ quy trình tải file đa luồng
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.downloaditem.http.HttpDownloadJob.resumeWithNewScope
    pub async fn start_download<F>(
        &self,
        task_id: String,
        url: String,
        headers: HashMap<String, String>,
        save_path: PathBuf,
        is_canceled: Arc<AtomicBool>,
        mut on_progress: F,
    ) -> Result<PathBuf, DownloadError>
    where
        F: FnMut(DownloadProgressEvent) + Send + 'static,
    {
        // 1. Thăm dò thông tin file (Probe)
        let prober = UrlProber::new(self.client.clone());
        let meta = prober.probe(&url, &headers).await?;

        let total_bytes = meta.content_length;
        let final_filename = meta
            .suggested_filename
            .unwrap_or_else(|| "download_file".to_string());

        let final_path = if save_path.is_dir() {
            save_path.join(&final_filename)
        } else {
            save_path
        };

        // File tạm trong quá trình tải
        let incomplete_path = final_path.with_extension(format!(
            "{}.part",
            final_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("tmp")
        ));

        // 2. Chia dải byte (Part Slicing)
        let threads = if meta.supports_range && total_bytes.is_some() {
            self.thread_count
        } else {
            1
        };

        let mut parts = Vec::new();
        if let Some(total) = total_bytes {
            let chunk_size = total / threads as u64;
            for i in 0..threads {
                let start = i as u64 * chunk_size;
                let end = if i == threads - 1 {
                    total - 1
                } else {
                    (start + chunk_size) - 1
                };
                parts.push(PartInfo {
                    index: i,
                    start_byte: start,
                    end_byte: end,
                    downloaded_bytes: 0,
                    is_completed: false,
                });
            }
        } else {
            // Server không báo trước kích thước file -> 1 part duy nhất
            parts.push(PartInfo {
                index: 0,
                start_byte: 0,
                end_byte: u64::MAX,
                downloaded_bytes: 0,
                is_completed: false,
            });
        }

        // 3. Chuẩn bị file và cấp phát Sparse File trên NTFS
        {
            let file = File::create(&incomplete_path)?;
            if let Some(total) = total_bytes {
                let _ = mark_as_sparse_file(&file);
                file.set_len(total)?;
            }
        }

        // 4. Điều phối tải song song các parts
        let downloaded_counter = Arc::new(AtomicU64::new(0));
        let mut part_handles = Vec::new();
        let downloader = Arc::new(HttpPartDownloader {
            client: self.client.clone(),
        });

        for mut part in parts {
            let dl = downloader.clone();
            let p_url = url.clone();
            let p_headers = headers.clone();
            let p_path = incomplete_path.clone();
            let p_counter = downloaded_counter.clone();
            let p_canceled = is_canceled.clone();

            let handle = tokio::spawn(async move {
                dl.download_part(
                    &p_url,
                    &p_headers,
                    &mut part,
                    p_path,
                    p_counter,
                    p_canceled,
                )
                .await
            });

            part_handles.push(handle);
        }

        // 5. Luồng đo tốc độ và phát tín hiệu tiến độ thời gian thực
        let monitor_counter = downloaded_counter.clone();
        let monitor_canceled = is_canceled.clone();
        let m_task_id = task_id.clone();

        let (progress_tx, mut progress_rx) = mpsc::channel::<DownloadProgressEvent>(100);

        let progress_task = tokio::spawn(async move {
            let mut meter = SpeedMeter::new(3);
            while !monitor_canceled.load(Ordering::Relaxed) {
                sleep(Duration::from_millis(300)).await;
                let downloaded = monitor_counter.load(Ordering::Relaxed);
                meter.update(downloaded);

                let speed = meter.current_speed_bps();
                let eta = meter.calculate_eta(downloaded, total_bytes);

                let event = DownloadProgressEvent {
                    id: m_task_id.clone(),
                    downloaded_bytes: downloaded,
                    total_bytes,
                    speed_bps: speed,
                    eta_seconds: eta,
                    status: DownloadStatus::Downloading,
                };

                if progress_tx.send(event).await.is_err() {
                    break;
                }

                if let Some(total) = total_bytes {
                    if downloaded >= total {
                        break;
                    }
                }
            }
        });

        // Vòng lặp nhận event tiến độ
        let event_loop = tokio::spawn(async move {
            while let Some(evt) = progress_rx.recv().await {
                on_progress(evt);
            }
        });

        // Chờ toàn bộ các worker parts hoàn thành
        for handle in part_handles {
            match handle.await {
                Ok(result) => result?,
                Err(_) => return Err(DownloadError::Canceled),
            }
        }

        let _ = progress_task.await;
        let _ = event_loop.await;

        if is_canceled.load(Ordering::Relaxed) {
            return Err(DownloadError::Canceled);
        }

        // 6. Hoàn tất tải: Đổi tên file từ .part sang tên chính thức
        std::fs::rename(&incomplete_path, &final_path)?;

        Ok(final_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_multi_part_download_coordinator() {
        let server = MockServer::start().await;

        let payload = "Flow Speed Link Multi-part High Speed Engine Content";
        let length = payload.len() as u64;

        // Mock HEAD request
        Mock::given(method("HEAD"))
            .and(path("/file.dat"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-length", length.to_string())
                    .insert_header("accept-ranges", "bytes"),
            )
            .mount(&server)
            .await;

        // Mock GET range request
        Mock::given(method("GET"))
            .and(path("/file.dat"))
            .respond_with(
                ResponseTemplate::new(206)
                    .set_body_bytes(payload.as_bytes())
                    .insert_header("content-length", length.to_string()),
            )
            .mount(&server)
            .await;

        let dir = tempdir().unwrap();
        let target_file = dir.path().join("file.dat");

        let coordinator = HttpDownloadCoordinator::new(2);
        let canceled = Arc::new(AtomicBool::new(false));

        let result = coordinator
            .start_download(
                "task-1".to_string(),
                format!("{}/file.dat", server.uri()),
                HashMap::new(),
                target_file.clone(),
                canceled,
                |_progress| {},
            )
            .await;

        assert!(result.is_ok());
        assert!(target_file.exists());
    }
}
