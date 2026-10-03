//! In-memory thread-safe rate limiting using token bucket algorithm.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Result of evaluating a rate limit policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitResult {
    /// Request is permitted under current quota.
    Allowed {
        /// Number of remaining tokens in current window.
        remaining: u64,
        /// Milliseconds until full bucket restoration.
        reset_after_ms: u64,
    },
    /// Request exceeds rate limit threshold.
    Exceeded {
        /// Suggested retry wait time in milliseconds.
        retry_after_ms: u64,
    },
}

/// Abstract rate limiter interface.
pub trait RateLimiter: Send + Sync {
    /// Checks quota for a given client identifier or key.
    fn check(&self, key: &str) -> RateLimitResult;
}

#[derive(Debug, Clone)]
struct BucketState {
    tokens: u64,
    last_update_ms: i64,
}

/// Thread-safe token bucket rate limiter with integer arithmetic.
pub struct TokenBucketLimiter {
    capacity: u64,
    refill_per_sec: u64,
    buckets: RwLock<HashMap<String, BucketState>>,
}

impl TokenBucketLimiter {
    /// Creates a new token bucket with specified capacity and refill rate per second.
    #[must_use]
    pub fn new(capacity: u64, refill_per_sec: u64) -> Self {
        Self {
            capacity: capacity.max(1),
            refill_per_sec: refill_per_sec.max(1),
            buckets: RwLock::new(HashMap::new()),
        }
    }
}

impl RateLimiter for TokenBucketLimiter {
    fn check(&self, key: &str) -> RateLimitResult {
        let now_ms = Utc::now().timestamp_millis();
        let mut buckets = self.buckets.write().unwrap();

        let state = buckets
            .entry(key.to_string())
            .or_insert_with(|| BucketState {
                tokens: self.capacity,
                last_update_ms: now_ms,
            });

        // Refill tokens based on integer elapsed milliseconds
        let elapsed_ms = (now_ms - state.last_update_ms).max(0) as u64;
        let tokens_to_add = (elapsed_ms * self.refill_per_sec) / 1000;

        if tokens_to_add > 0 {
            state.tokens = (state.tokens + tokens_to_add).min(self.capacity);
            state.last_update_ms = now_ms;
        }

        if state.tokens >= 1 {
            state.tokens -= 1;
            let missing_tokens = self.capacity.saturating_sub(state.tokens);
            let reset_after_ms = (missing_tokens * 1000) / self.refill_per_sec;

            RateLimitResult::Allowed {
                remaining: state.tokens,
                reset_after_ms,
            }
        } else {
            let retry_after_ms = (1000 / self.refill_per_sec).max(1);
            RateLimitResult::Exceeded { retry_after_ms }
        }
    }
}
