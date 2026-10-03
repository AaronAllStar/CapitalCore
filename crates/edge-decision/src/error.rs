//! Error types for decision engine evaluation and audit persistence.

use thiserror::Error;

/// Error domain for decision engine execution.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DecisionError {
    /// Failed to append decision audit record.
    #[error("audit log error: {0}")]
    Audit(String),

    /// Evaluation failure during decision processing.
    #[error("evaluation error: {0}")]
    Evaluation(String),
}
