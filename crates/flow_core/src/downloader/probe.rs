use reqwest::header::{ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH, ETAG, LAST_MODIFIED, RANGE};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UrlMetadata {
    pub content_length: Option<u64>,
    pub supports_range: bool,
    pub suggested_filename: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

pub struct UrlProber {
    client: Client,
}

impl UrlProber {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Thăm dò thông tin URL để lấy kích thước file, hỗ trợ tải đa luồng và tên file gợi ý
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.downloaditem.http.HttpDownloadJob.fetchDownloadInfoAndValidate
    pub async fn probe(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<UrlMetadata, reqwest::Error> {
        // Thử HEAD request trước
        let mut req = self.client.head(url);
        for (k, v) in headers {
            req = req.header(k, v);
        }

        let resp_result = req.send().await;

        let resp = match resp_result {
            Ok(r) if r.status().is_success() => r,
            _ => {
                // Nếu HEAD bị từ chối hoặc lỗi, gửi GET với Range: bytes=0-0 (chuẩn probe của IDM)
                let mut get_req = self.client.get(url).header(RANGE, "bytes=0-0");
                for (k, v) in headers {
                    get_req = get_req.header(k, v);
                }
                get_req.send().await?
            }
        };

        let headers_ref = resp.headers();

        // Kiểm tra hỗ trợ range
        let supports_range = if let Some(val) = headers_ref.get(ACCEPT_RANGES) {
            val.to_str().unwrap_or("").to_lowercase().contains("bytes")
        } else {
            resp.status().as_u16() == 206
        };

        // Lấy Content-Length
        let content_length = if let Some(val) = headers_ref.get(CONTENT_LENGTH) {
            val.to_str().ok().and_then(|s| s.parse::<u64>().ok())
        } else {
            None
        };

        // Lấy ETag & Last-Modified
        let etag = headers_ref
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let last_modified = headers_ref
            .get(LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Lấy tên file từ Content-Disposition
        let suggested_filename = if let Some(disp) = headers_ref.get(CONTENT_DISPOSITION) {
            disp.to_str().ok().and_then(parse_content_disposition_filename)
        } else {
            None
        }
        .or_else(|| extract_filename_from_url(url));

        Ok(UrlMetadata {
            content_length,
            supports_range,
            suggested_filename,
            etag,
            last_modified,
        })
    }
}

/// Trích xuất tên file từ header Content-Disposition
fn parse_content_disposition_filename(disposition: &str) -> Option<String> {
    for part in disposition.split(';') {
        let trimmed = part.trim();
        if let Some(rest) = trimmed.strip_prefix("filename=") {
            let clean = rest.trim_matches('"').trim();
            if !clean.is_empty() {
                return Some(clean.to_string());
            }
        }
    }
    None
}

/// Trích xuất tên file từ đường dẫn URL
fn extract_filename_from_url(url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url).ok()?;
    let path = parsed.path();
    let name = path.split('/').last()?.trim();
    if !name.is_empty() {
        Some(name.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename_from_url() {
        assert_eq!(
            extract_filename_from_url("https://example.com/downloads/setup.exe?key=123"),
            Some("setup.exe".to_string())
        );
        assert_eq!(
            extract_filename_from_url("https://example.com/files/archive.tar.gz"),
            Some("archive.tar.gz".to_string())
        );
    }

    #[test]
    fn test_parse_content_disposition() {
        let header = "attachment; filename=\"report_2026.pdf\"";
        assert_eq!(
            parse_content_disposition_filename(header),
            Some("report_2026.pdf".to_string())
        );
    }
}
