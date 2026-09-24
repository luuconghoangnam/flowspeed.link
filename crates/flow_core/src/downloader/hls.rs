use crate::downloader::part::DownloadError;
use m3u8_rs::playlist::Playlist;
use reqwest::Client;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub struct HlsDownloader {
    pub client: Client,
}

impl HlsDownloader {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Phân tích playlist M3U8 và tải tuần tự/song song các segments
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.download.HLSDownloadJob
    pub async fn download_hls(
        &self,
        m3u8_url: &str,
        save_path: impl AsRef<Path>,
        downloaded_counter: Arc<AtomicU64>,
        is_canceled: Arc<AtomicBool>,
    ) -> Result<(), DownloadError> {
        let resp = self.client.get(m3u8_url).send().await?;
        let bytes = resp.bytes().await?;

        let playlist = match m3u8_rs::parse_playlist_res(&bytes) {
            Ok(p) => p,
            Err(_) => return Err(DownloadError::RangeNotSupported),
        };

        let media_playlist = match playlist {
            Playlist::MasterPlaylist(_master) => {
                // TODO: Chọn stream có bitrate cao nhất từ master playlist
                return Ok(());
            }
            Playlist::MediaPlaylist(media) => media,
        };

        let mut output_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(save_path)
            .await?;

        for segment in media_playlist.segments {
            if is_canceled.load(Ordering::Relaxed) {
                return Err(DownloadError::Canceled);
            }

            // Xử lý absolute / relative URI của segment
            let segment_url = if segment.uri.starts_with("http://") || segment.uri.starts_with("https://") {
                segment.uri
            } else {
                let base = reqwest::Url::parse(m3u8_url).map_err(|_| DownloadError::RangeNotSupported)?;
                base.join(&segment.uri)
                    .map_err(|_| DownloadError::RangeNotSupported)?
                    .to_string()
            };

            let seg_resp = self.client.get(&segment_url).send().await?;
            let seg_bytes = seg_resp.bytes().await?;

            output_file.write_all(&seg_bytes).await?;
            downloaded_counter.fetch_add(seg_bytes.len() as u64, Ordering::Relaxed);
        }

        output_file.flush().await?;
        Ok(())
    }
}
