//! Error types for rule management, serialization, and persistence.

use edge_domain::RuleId;
use thiserror::Error;

/// Error domain for rule definitions and repository operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuleError {
    /// Rule with specified identifier was not found.
    #[error("rule with id '{0}' not found")]
    NotFound(RuleId),

    /// Rule with specified identifier already exists.
    #[error("rule with id '{0}' already exists")]
    Duplicate(RuleId),

    /// Rule serialization or deserialization failure.
    #[error("rule serialization error: {0}")]
    Serialization(String),

    /// Underlying storage or lock error.
    #[error("storage error: {0}")]
    Storage(String),
}
