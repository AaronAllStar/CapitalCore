use thiserror::Error;

/// Errors arising during event dispatch and handling.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EventError {
    #[error("handler '{handler_name}' failed: {reason}")]
    HandlerFailed {
        handler_name: String,
        reason: String,
    },

    #[error("event handling timed out after {0:?}")]
    Timeout(std::time::Duration),

    #[error("event bus is closed")]
    BusClosed,
}
