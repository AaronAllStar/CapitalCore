use edge_core::NumericError;
use thiserror::Error;

/// Errors arising within the core financial domain.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("required domain field is missing: {0}")]
    MissingField(&'static str),

    #[error("invalid domain state: {0}")]
    InvalidState(String),

    #[error("numeric computation error: {0}")]
    Numeric(#[from] NumericError),
}
