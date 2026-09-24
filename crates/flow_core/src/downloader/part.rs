use crate::types::PartInfo;
use futures_util::StreamExt;
use reqwest::header::RANGE;
use reqwest::Client;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Canceled by user")]
    Canceled,
    #[error("Server does not support range requests")]
    RangeNotSupported,
}

pub struct HttpPartDownloader {
    pub client: Client,
}

impl HttpPartDownloader {
    pub fn new() -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .tcp_nodelay(true)
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// Tải một part cụ thể theo dải byte (Range: bytes=start-end) và ghi trực tiếp vào offset tương ứng của file
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.part.HttpPartDownloader
    pub async fn download_part(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        part: &mut PartInfo,
        save_path: impl AsRef<Path>,
        downloaded_counter: Arc<AtomicU64>,
        is_canceled: Arc<AtomicBool>,
    ) -> Result<(), DownloadError> {
        let current_start = part.start_byte + part.downloaded_bytes;
        if current_start > part.end_byte {
            part.is_completed = true;
            return Ok(());
        }

        let mut req = self.client.get(url);
        for (k, v) in headers {
            req = req.header(k, v);
        }

        let range_val = format!("bytes={}-{}", current_start, part.end_byte);
        req = req.header(RANGE, range_val);

        let resp = req.send().await?;
        if !resp.status().is_success() && resp.status().as_u16() != 206 {
            return Err(DownloadError::RangeNotSupported);
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(save_path)
            .await?;

        file.seek(SeekFrom::Start(current_start)).await?;

        let mut stream = resp.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            if is_canceled.load(Ordering::Relaxed) {
                return Err(DownloadError::Canceled);
            }

            let chunk = chunk_result?;
            file.write_all(&chunk).await?;

            let bytes_len = chunk.len() as u64;
            part.downloaded_bytes += bytes_len;
            downloaded_counter.fetch_add(bytes_len, Ordering::Relaxed);
        }

        file.flush().await?;
        if part.start_byte + part.downloaded_bytes > part.end_byte {
            part.is_completed = true;
        }

        Ok(())
    }
}
