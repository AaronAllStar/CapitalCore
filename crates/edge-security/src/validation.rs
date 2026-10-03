//! Input validation, boundary checking, and string sanitization.

use thiserror::Error;

/// Input validation error domain.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// Field was unexpectedly empty or whitespace only.
    #[error("field '{0}' cannot be empty")]
    EmptyField(String),

    /// Field length exceeded allowed upper bound.
    #[error("field '{field}' exceeds maximum length of {max} (was {actual})")]
    TooLong {
        /// Field identifier.
        field: String,
        /// Maximum permitted length.
        max: usize,
        /// Actual observed length.
        actual: usize,
    },

    /// Field length was below the required lower bound.
    #[error("field '{field}' is below minimum length of {min} (was {actual})")]
    TooShort {
        /// Field identifier.
        field: String,
        /// Minimum permitted length.
        min: usize,
        /// Actual observed length.
        actual: usize,
    },

    /// Field does not conform to required syntax or schema.
    #[error("field '{field}' has invalid format: {message}")]
    InvalidFormat {
        /// Field identifier.
        field: String,
        /// Explanation of format mismatch.
        message: String,
    },

    /// Value is outside acceptable domain bounds.
    #[error("field '{field}' has invalid value: {message}")]
    InvalidValue {
        /// Field identifier.
        field: String,
        /// Explanation of invalidity.
        message: String,
    },
}

/// Trait implemented by request and data structures that can be validated.
pub trait Validate {
    /// Performs validation and returns `Ok(())` or the first `ValidationError`.
    fn validate(&self) -> Result<(), ValidationError>;
}

/// Validates that a string slice is not empty or composed solely of whitespace.
pub fn validate_not_empty(field: &str, value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::EmptyField(field.to_string()))
    } else {
        Ok(())
    }
}

/// Validates that a string length falls within `[min, max]`.
pub fn validate_length(
    field: &str,
    value: &str,
    min: usize,
    max: usize,
) -> Result<(), ValidationError> {
    let len = value.chars().count();
    if len < min {
        Err(ValidationError::TooShort {
            field: field.to_string(),
            min,
            actual: len,
        })
    } else if len > max {
        Err(ValidationError::TooLong {
            field: field.to_string(),
            max,
            actual: len,
        })
    } else {
        Ok(())
    }
}

/// Validates basic email formatting rules (single '@', non-empty user and domain).
pub fn validate_email(field: &str, email: &str) -> Result<(), ValidationError> {
    validate_not_empty(field, email)?;
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() || !parts[1].contains('.') {
        return Err(ValidationError::InvalidFormat {
            field: field.to_string(),
            message: "must be a valid email address with user and domain".to_string(),
        });
    }
    Ok(())
}

/// Strips dangerous ASCII control characters while preserving standard whitespace.
#[must_use]
pub fn sanitize_control_chars(input: &str) -> String {
    input
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
        .collect()
}
