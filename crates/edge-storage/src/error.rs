//! Storage and persistence error definitions.

use thiserror::Error;

/// Storage and persistence error domain.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StorageError {
    /// Generic or low-level database error.
    #[error("database error: {0}")]
    Database(String),

    /// Requested record could not be found.
    #[error("record not found: {0}")]
    NotFound(String),

    /// Unique constraint or duplicate primary key conflict.
    #[error("duplicate key violation: {0}")]
    Duplicate(String),

    /// Database migration execution failed.
    #[error("migration execution failed: {0}")]
    Migration(String),

    /// Connection pool configuration or lifecycle error.
    #[error("connection pool configuration error: {0}")]
    PoolConfiguration(String),
}
