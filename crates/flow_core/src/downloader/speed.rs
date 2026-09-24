use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Bộ tính toán tốc độ tải mượt mà (Moving Average Speed Meter)
/// Sử dụng sliding window để đo chính xác bytes/giây và thời gian hoàn thành (ETA)
pub struct SpeedMeter {
    window_duration: Duration,
    samples: VecDeque<(Instant, u64)>, // (Thời điểm, Tổng số bytes đã tải lũy kế)
}

impl SpeedMeter {
    pub fn new(window_seconds: u64) -> Self {
        Self {
            window_duration: Duration::from_secs(window_seconds.max(1)),
            samples: VecDeque::new(),
        }
    }

    /// Thêm một mẫu dữ liệu đo lường mới
    pub fn update(&mut self, total_bytes: u64) {
        let now = Instant::now();
        self.samples.push_back((now, total_bytes));

        // Xóa các mẫu nằm ngoài cửa sổ thời gian
        while let Some(&(time, _)) = self.samples.front() {
            if time.elapsed() > self.window_duration {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }

    /// Tốc độ hiện tại tính theo Bytes/giây (Bps)
    pub fn current_speed_bps(&self) -> u64 {
        if self.samples.len() < 2 {
            return 0;
        }

        let (first_time, first_bytes) = self.samples.front().unwrap();
        let (last_time, last_bytes) = self.samples.back().unwrap();

        let duration = last_time.saturating_duration_since(*first_time).as_secs_f64();
        if duration <= 0.001 {
            return 0;
        }

        let bytes_diff = last_bytes.saturating_sub(*first_bytes);
        (bytes_diff as f64 / duration) as u64
    }

    /// Thời gian dự kiến hoàn thành tính theo giây (ETA)
    pub fn calculate_eta(&self, downloaded_bytes: u64, total_bytes: Option<u64>) -> Option<u64> {
        let total = total_bytes?;
        if downloaded_bytes >= total {
            return Some(0);
        }

        let speed = self.current_speed_bps();
        if speed == 0 {
            return None; // Không xác định được ETA nếu tốc độ = 0
        }

        let remaining = total - downloaded_bytes;
        Some(remaining / speed)
    }

    /// Định dạng tốc độ sang dạng đọc dễ hiểu (ví dụ: "45.2 MB/s", "820 KB/s")
    pub fn format_speed(bps: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = 1024.0 * 1024.0;
        const GB: f64 = 1024.0 * 1024.0 * 1024.0;

        let b = bps as f64;
        if b >= GB {
            format!("{:.2} GB/s", b / GB)
        } else if b >= MB {
            format!("{:.2} MB/s", b / MB)
        } else if b >= KB {
            format!("{:.1} KB/s", b / KB)
        } else {
            format!("{} B/s", bps)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_speed() {
        assert_eq!(SpeedMeter::format_speed(500), "500 B/s");
        assert_eq!(SpeedMeter::format_speed(1536), "1.5 KB/s");
        assert_eq!(SpeedMeter::format_speed(45 * 1024 * 1024), "45.00 MB/s");
    }

    #[test]
    fn test_eta_calculation() {
        let mut meter = SpeedMeter::new(2);
        let now = Instant::now();
        meter.samples.push_back((now, 1000));
        meter.samples.push_back((now + Duration::from_secs(1), 2000));

        let eta = meter.calculate_eta(2000, Some(5000));
        assert_eq!(eta, Some(3)); // Còn lại 3000 bytes, tốc độ 1000 bytes/s -> 3 giây
    }
}
