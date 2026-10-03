//! EdgeArena security layer: input validation, rate limiting, and error formatting.
//!
//! Enforces boundary constraints, protects against volumetric abuse via token bucket rate limiting,
//! and standardizes JSON error envelopes across the platform.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error_response;
pub mod headers;
pub mod rate_limiter;
pub mod scrubber;
pub mod validation;

pub use error_response::ErrorResponse;
pub use headers::standard_security_headers;
pub use rate_limiter::{RateLimitResult, RateLimiter, TokenBucketLimiter};
pub use scrubber::{scrub_json_value, scrub_string};
pub use validation::{
    sanitize_control_chars, validate_email, validate_length, validate_not_empty, Validate,
    ValidationError,
};

/// Compares two byte slices in constant time to prevent timing side-channel attacks.
#[must_use]
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

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

    #[test]
    fn test_secret_scrubbing_string() {
        let text = "Log info: user provided Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.fake and card 4111 2222 3333 4444 with \"password\": \"supersecret123\"";
        let scrubbed = scrub_string(text);

        assert!(!scrubbed.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.fake"));
        assert!(!scrubbed.contains("4111 2222 3333 4444"));
        assert!(!scrubbed.contains("supersecret123"));
        assert!(scrubbed.contains("[REDACTED_TOKEN]"));
        assert!(scrubbed.contains("[REDACTED_PAN]"));
        assert!(scrubbed.contains(r#""password": "[REDACTED]""#));
    }

    #[test]
    fn test_secret_scrubbing_json() {
        let mut json = serde_json::json!({
            "user": "alice",
            "password": "mypassword",
            "metadata": {
                "api_key": "secret_key_123456789",
                "nested_token": "Bearer eyJhbGciOiJIUzI1NiJ9.abc.def"
            }
        });

        scrub_json_value(&mut json);
        assert_eq!(json["password"], "[REDACTED]");
        assert_eq!(json["metadata"]["api_key"], "[REDACTED]");
        assert_eq!(json["metadata"]["nested_token"], "[REDACTED]");
        assert_eq!(json["user"], "alice");
    }

    #[test]
    fn test_standard_security_headers() {
        let headers = standard_security_headers();
        assert_eq!(headers.len(), 7);
        assert!(headers
            .iter()
            .any(|(k, v)| *k == "x-frame-options" && *v == "DENY"));
        assert!(headers
            .iter()
            .any(|(k, v)| *k == "x-content-type-options" && *v == "nosniff"));
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare(b"password123", b"password123"));
        assert!(!constant_time_compare(b"password123", b"password124"));
        assert!(!constant_time_compare(b"short", b"longer_string"));
        assert!(constant_time_compare(b"", b""));
    }
}
