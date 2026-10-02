use crate::currency::Currency;
use thiserror::Error;

/// Errors arising from monetary and numeric operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NumericError {
    #[error("arithmetic overflow in operation: {0}")]
    Overflow(&'static str),

    #[error("arithmetic underflow in operation: {0}")]
    Underflow(&'static str),

    #[error("currency mismatch: expected {expected}, found {found}")]
    CurrencyMismatch { expected: Currency, found: Currency },

    #[error("division by zero")]
    DivisionByZero,

    #[error("invalid precision: {0}")]
    InvalidPrecision(String),

    #[error("failed to parse numeric value: {0}")]
    ParseError(String),
}
