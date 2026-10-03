//! Error definitions for the background worker subsystem.

use thiserror::Error;

/// Errors that can occur during worker task ingestion, execution, and retries.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum WorkerError {
    /// Handler encountered a transient execution error.
    #[error("transient handler failure: {0}")]
    Transient(String),

    /// Non-retryable poison message or unrecoverable error.
    #[error("non-retryable poison message: {0}")]
    PoisonMessage(String),

    /// Maximum configured retry attempts exceeded.
    #[error("max retry attempts ({attempts}) exceeded: {reason}")]
    MaxRetriesExceeded {
        /// Number of attempts made before failing.
        attempts: u32,
        /// Root cause description.
        reason: String,
    },

    /// Worker queue channel closed.
    #[error("worker queue channel closed")]
    QueueClosed,

    /// Dead letter queue storage error.
    #[error("dead letter queue error: {0}")]
    DlqError(String),
}
