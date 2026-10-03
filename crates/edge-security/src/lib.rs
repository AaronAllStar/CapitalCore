//! EdgeArena security layer: input validation, rate limiting, and error formatting.
//!
//! Enforces boundary constraints, protects against volumetric abuse via token bucket rate limiting,
//! and standardizes JSON error envelopes across the platform.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error_response;
pub mod rate_limiter;
pub mod validation;

pub use error_response::ErrorResponse;
pub use rate_limiter::{RateLimitResult, RateLimiter, TokenBucketLimiter};
pub use validation::{
    sanitize_control_chars, validate_email, validate_length, validate_not_empty, Validate,
    ValidationError,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_not_empty() {
        assert!(validate_not_empty("name", "Valid Name").is_ok());
        assert!(validate_not_empty("name", "").is_err());
        assert!(validate_not_empty("name", "   \t \n").is_err());
    }

    #[test]
    fn test_validation_length() {
        assert!(validate_length("code", "USD", 3, 3).is_ok());
        assert!(validate_length("code", "US", 3, 3).is_err());
        assert!(validate_length("code", "USDT", 3, 3).is_err());
    }

    #[test]
    fn test_validation_email() {
        assert!(validate_email("email", "trader@edgearena.io").is_ok());
        assert!(validate_email("email", "invalid_email").is_err());
        assert!(validate_email("email", "@missinguser.com").is_err());
        assert!(validate_email("email", "user@nodomain").is_err());
    }

    #[test]
    fn test_sanitize_control_chars() {
        let input = "hello\x00\x07world\nclean\ttab";
        let sanitized = sanitize_control_chars(input);
        assert_eq!(sanitized, "helloworld\nclean\ttab");
    }

    #[test]
    fn test_rate_limiter_token_bucket() {
        let limiter = TokenBucketLimiter::new(2, 10);
        let key = "client_ip_192.168.1.1";

        let r1 = limiter.check(key);
        match r1 {
            RateLimitResult::Allowed { remaining, .. } => assert_eq!(remaining, 1),
            RateLimitResult::Exceeded { .. } => panic!("expected allowed"),
        }

        let r2 = limiter.check(key);
        match r2 {
            RateLimitResult::Allowed { remaining, .. } => assert_eq!(remaining, 0),
            RateLimitResult::Exceeded { .. } => panic!("expected allowed"),
        }

        let r3 = limiter.check(key);
        match r3 {
            RateLimitResult::Allowed { .. } => panic!("expected exceeded"),
            RateLimitResult::Exceeded { retry_after_ms } => assert!(retry_after_ms > 0),
        }
    }

    #[test]
    fn test_error_response_builders() {
        let resp = ErrorResponse::bad_request("missing payload")
            .with_request_id("req-12345")
            .with_details(serde_json::json!({ "field": "amount" }));

        assert_eq!(resp.error_code, "ERR_BAD_REQUEST");
        assert_eq!(resp.request_id.as_deref(), Some("req-12345"));
        assert!(resp.details.is_some());

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("ERR_BAD_REQUEST"));
        assert!(json.contains("req-12345"));
    }
}
