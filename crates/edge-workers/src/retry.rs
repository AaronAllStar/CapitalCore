//! Configurable exponential backoff retry strategy.

use std::time::Duration;

/// Policy dictating backoff timings and maximum retry attempts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts permitted before routing to DLQ.
    pub max_retries: u32,
    /// Base delay duration before the first retry attempt.
    pub initial_backoff: Duration,
    /// Cap on exponential backoff duration.
    pub max_backoff: Duration,
    /// Backoff multiplier factor (e.g. 2 for binary exponential backoff).
    pub multiplier: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(50),
            max_backoff: Duration::from_secs(5),
            multiplier: 2,
        }
    }
}

impl RetryPolicy {
    /// Constructs a new custom `RetryPolicy`.
    #[must_use]
    pub fn new(
        max_retries: u32,
        initial_backoff: Duration,
        max_backoff: Duration,
        multiplier: u32,
    ) -> Self {
        Self {
            max_retries,
            initial_backoff,
            max_backoff,
            multiplier,
        }
    }

    /// Calculates the deterministic backoff delay for a given 1-based attempt index.
    #[must_use]
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let power = attempt.saturating_sub(1);
        let factor = (self.multiplier as u64).saturating_pow(power);
        let base_nanos = self.initial_backoff.as_nanos();
        let computed_nanos =
            (base_nanos.saturating_mul(factor as u128)).min(self.max_backoff.as_nanos());

        Duration::from_nanos(computed_nanos as u64)
    }

    /// Determines whether another retry should be attempted.
    #[must_use]
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}
