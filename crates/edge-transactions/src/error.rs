//! Transaction processing and ingestion error types.

use edge_security::ValidationError;
use thiserror::Error;

/// Error domain for transaction parsing, normalization, and ingestion.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TransactionError {
    /// Input validation failed.
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Duplicate event submission detected.
    #[error("duplicate event: {0}")]
    Duplicate(String),

    /// Normalization or parsing failure for incoming payload.
    #[error("normalization error: {0}")]
    Normalization(String),

    /// Failed to record audit log record during ingestion.
    #[error("audit logging error: {0}")]
    Audit(String),

    /// Failed to publish event to event bus.
    #[error("event bus dispatch error: {0}")]
    Bus(String),
}
