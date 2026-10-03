//! Standardized API error responses following structured problem envelope format.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Standardized JSON error response envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Machine-readable alphanumeric error identifier (e.g. "ERR_BAD_REQUEST").
    pub error_code: String,
    /// Human-readable explanation of error condition.
    pub message: String,
    /// Optional structured debugging or validation details.
    pub details: Option<serde_json::Value>,
    /// UTC timestamp when error occurred.
    pub timestamp: DateTime<Utc>,
    /// Optional correlation identifier for distributed tracing.
    pub request_id: Option<String>,
}

impl ErrorResponse {
    /// Creates a base error response with current timestamp.
    #[must_use]
    pub fn new(error_code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_code: error_code.into(),
            message: message.into(),
            details: None,
            timestamp: Utc::now(),
            request_id: None,
        }
    }

    /// Attaches structured error details.
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Attaches a request correlation ID.
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Creates a 400 Bad Request error response.
    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("ERR_BAD_REQUEST", message)
    }

    /// Creates a 401 Unauthorized error response.
    #[must_use]
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("ERR_UNAUTHORIZED", message)
    }

    /// Creates a 403 Forbidden error response.
    #[must_use]
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new("ERR_FORBIDDEN", message)
    }

    /// Creates a 404 Not Found error response.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("ERR_NOT_FOUND", message)
    }

    /// Creates a 429 Too Many Requests error response.
    #[must_use]
    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::new("ERR_TOO_MANY_REQUESTS", message)
    }

    /// Creates a 500 Internal Server Error response.
    #[must_use]
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("ERR_INTERNAL_ERROR", message)
    }
}
