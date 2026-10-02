use thiserror::Error;

/// Errors arising from audit logging and verification operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuditError {
    #[error("audit entry not found for identifier: {0}")]
    EntryNotFound(String),

    #[error("audit storage failed: {0}")]
    StorageFailed(String),

    #[error("audit log integrity verification failed at sequence {sequence}: expected hash {expected}, computed {computed}")]
    IntegrityViolation {
        sequence: u64,
        expected: String,
        computed: String,
    },
}
