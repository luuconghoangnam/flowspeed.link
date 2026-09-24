use crate::downloader::part::DownloadError;
use crate::downloader::speed::SpeedMeter;
use crate::types::{DownloadProgressEvent, DownloadStatus};
use m3u8_rs::Playlist;
use reqwest::Client;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::time::sleep;

pub struct HlsDownloader {
    pub client: Client,
    pub max_concurrent_segments: usize,
}

impl Default for HlsDownloader {
    fn default() -> Self {
        Self::new(4)
    }
}

impl HlsDownloader {
    pub fn new(max_concurrent_segments: usize) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(20)
            .tcp_nodelay(true)
            .build()
            .unwrap_or_default();

        Self {
            client,
            max_concurrent_segments: max_concurrent_segments.max(1),
        }
    }

    /// Tải toàn bộ video HLS M3U8 (hỗ trợ cả Master Playlist & Media Playlist)
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.downloaditem.hls.HLSDownloadJob
    pub async fn download_hls<F>(
        &self,
        task_id: String,
        m3u8_url: &str,
        headers: &HashMap<String, String>,
        save_path: impl AsRef<Path>,
        is_canceled: Arc<AtomicBool>,
        mut on_progress: F,
    ) -> Result<PathBuf, DownloadError>
    where
        F: FnMut(DownloadProgressEvent) + Send + 'static,
    {
        let final_path = save_path.as_ref().to_path_buf();
        let incomplete_path = final_path.with_extension(format!(
            "{}.part",
            final_path.extension().and_then(|s| s.to_str()).unwrap_or("ts")
        ));

        // 1. Tải và phân tích file playlist ban đầu
        let mut req = self.client.get(m3u8_url);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let resp = req.send().await?;
        let bytes = resp.bytes().await?;

        let playlist = match m3u8_rs::parse_playlist_res(&bytes) {
            Ok(p) => p,
            Err(_) => return Err(DownloadError::RangeNotSupported),
        };

        // 2. Xử lý Master Playlist: Chọn Stream có bitrate (bandwidth) cao nhất
        let (media_playlist, base_url) = match playlist {
            Playlist::MasterPlaylist(master) => {
                let best_variant = master
                    .variants
                    .iter()
                    .max_by_key(|v| v.bandwidth)
                    .ok_or(DownloadError::RangeNotSupported)?;

                let variant_url = resolve_uri(m3u8_url, &best_variant.uri)?;

                let mut v_req = self.client.get(&variant_url);
                for (k, v) in headers {
                    v_req = v_req.header(k, v);
                }
                let v_resp = v_req.send().await?;
                let v_bytes = v_resp.bytes().await?;

                let v_playlist = match m3u8_rs::parse_playlist_res(&v_bytes) {
                    Ok(Playlist::MediaPlaylist(media)) => media,
                    _ => return Err(DownloadError::RangeNotSupported),
                };

                (v_playlist, variant_url)
            }
            Playlist::MediaPlaylist(media) => (media, m3u8_url.to_string()),
        };

        let total_segments = media_playlist.segments.len();
        if total_segments == 0 {
            return Err(DownloadError::RangeNotSupported);
        }

        // 3. Chuẩn bị file đầu ra
        let mut output_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&incomplete_path)
            .await?;

        let downloaded_counter = Arc::new(AtomicU64::new(0));

        // 4. Bật luồng theo dõi tiến độ thời gian thực
        let monitor_counter = downloaded_counter.clone();
        let monitor_canceled = is_canceled.clone();
        let m_task_id = task_id.clone();

        let (progress_tx, mut progress_rx) = mpsc::channel::<DownloadProgressEvent>(50);

        let is_finished = Arc::new(AtomicBool::new(false));
        let monitor_finished = is_finished.clone();

        let progress_task = tokio::spawn(async move {
            let mut meter = SpeedMeter::new(3);
            while !monitor_canceled.load(Ordering::Relaxed) && !monitor_finished.load(Ordering::Relaxed) {
                sleep(Duration::from_millis(300)).await;
                let downloaded = monitor_counter.load(Ordering::Relaxed);
                meter.update(downloaded);

                let speed = meter.current_speed_bps();

                let event = DownloadProgressEvent {
                    id: m_task_id.clone(),
                    downloaded_bytes: downloaded,
                    total_bytes: None, // HLS stream thường không cố định byte trước khi tải hết
                    speed_bps: speed,
                    eta_seconds: None,
                    status: DownloadStatus::Downloading,
                };

                if progress_tx.send(event).await.is_err() {
                    break;
                }
            }
        });

        let event_loop = tokio::spawn(async move {
            while let Some(evt) = progress_rx.recv().await {
                on_progress(evt);
            }
        });

        // 5. Tải tuần tự các segments và ghi trực tiếp vào file video
        for segment in media_playlist.segments {
            if is_canceled.load(Ordering::Relaxed) {
                return Err(DownloadError::Canceled);
            }

            let segment_url = resolve_uri(&base_url, &segment.uri)?;

            let mut s_req = self.client.get(&segment_url);
            for (k, v) in headers {
                s_req = s_req.header(k, v);
            }
            let seg_resp = s_req.send().await?;
            let seg_bytes = seg_resp.bytes().await?;

            output_file.write_all(&seg_bytes).await?;
            downloaded_counter.fetch_add(seg_bytes.len() as u64, Ordering::Relaxed);
        }

        output_file.flush().await?;
        is_finished.store(true, Ordering::Relaxed);

        let _ = progress_task.await;
        let _ = event_loop.await;

        if is_canceled.load(Ordering::Relaxed) {
            return Err(DownloadError::Canceled);
        }

        // 6. Hoàn tất: đổi tên file từ .part sang file chính thức
        std::fs::rename(&incomplete_path, &final_path)?;

        Ok(final_path)
    }
}

/// Chuẩn hóa đường dẫn tương đối (Relative URI) thành tuyệt đối (Absolute URL)
fn resolve_uri(base: &str, uri: &str) -> Result<String, DownloadError> {
    if uri.starts_with("http://") || uri.starts_with("https://") {
        Ok(uri.to_string())
    } else {
        let base_url = reqwest::Url::parse(base).map_err(|_| DownloadError::RangeNotSupported)?;
        let resolved = base_url
            .join(uri)
            .map_err(|_| DownloadError::RangeNotSupported)?;
        Ok(resolved.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_hls_stream_downloader() {
        let server = MockServer::start().await;

        // Sample Media Playlist
        let m3u8_content = "#EXTM3U\n#EXT-X-TARGETDURATION:10\n#EXTINF:9.009,\nseg0.ts\n#EXTINF:9.009,\nseg1.ts\n#EXT-X-ENDLIST\n";

        Mock::given(method("GET"))
            .and(path("/stream.m3u8"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(m3u8_content)
                    .insert_header("content-type", "application/vnd.apple.mpegurl"),
            )
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/seg0.ts"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"VIDEO_CHUNK_0"))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/seg1.ts"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"VIDEO_CHUNK_1"))
            .mount(&server)
            .await;

        let dir = tempdir().unwrap();
        let target_video = dir.path().join("output.ts");

        let downloader = HlsDownloader::new(2);
        let canceled = Arc::new(AtomicBool::new(false));

        let result = downloader
            .download_hls(
                "hls-1".to_string(),
                &format!("{}/stream.m3u8", server.uri()),
                &HashMap::new(),
                target_video.clone(),
                canceled,
                |_progress| {},
            )
            .await;

        assert!(result.is_ok());
        assert!(target_video.exists());

        let content = std::fs::read(&target_video).unwrap();
        assert_eq!(content, b"VIDEO_CHUNK_0VIDEO_CHUNK_1");
    }
}
