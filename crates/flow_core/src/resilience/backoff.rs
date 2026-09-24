use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HttpRetryPolicy {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for HttpRetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(30000),
            backoff_multiplier: 2.0,
        }
    }
}

impl HttpRetryPolicy {
    /// Tính toán thời gian delay theo thuật toán Exponential Backoff
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.retry.HttpRetryPolicy
    pub fn calculate_backoff_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }
        let exp = attempt.saturating_sub(1);
        let multiplier = self.backoff_multiplier.powi(exp as i32);
        let delay_ms = (self.initial_delay.as_millis() as f64 * multiplier) as u64;
        let max_ms = self.max_delay.as_millis() as u64;
        Duration::from_millis(delay_ms.min(max_ms))
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff_calculation() {
        let policy = HttpRetryPolicy::default();
        assert_eq!(policy.calculate_backoff_delay(0), Duration::from_millis(0));
        assert_eq!(policy.calculate_backoff_delay(1), Duration::from_millis(1000));
        assert_eq!(policy.calculate_backoff_delay(2), Duration::from_millis(2000));
        assert_eq!(policy.calculate_backoff_delay(3), Duration::from_millis(4000));
        assert_eq!(policy.calculate_backoff_delay(4), Duration::from_millis(8000));
        assert_eq!(policy.calculate_backoff_delay(5), Duration::from_millis(16000));
        assert_eq!(policy.calculate_backoff_delay(6), Duration::from_millis(30000)); // Capped at max_delay
    }
}
