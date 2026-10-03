//! Authentication and authorization error types.

use thiserror::Error;

/// Error domain for authentication, token handling, and authorization checks.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuthError {
    /// Token format is malformed or missing components.
    #[error("invalid token: {0}")]
    InvalidToken(String),

    /// Token expiration timestamp has elapsed.
    #[error("token has expired")]
    ExpiredToken,

    /// Cryptographic signature verification failed.
    #[error("invalid token signature")]
    InvalidSignature,

    /// Password hashing or verification failure.
    #[error("password hashing error: {0}")]
    PasswordHashing(String),

    /// User credentials failed verification.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// Access denied due to insufficient permissions.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// Key encoding, decoding, or generation failure.
    #[error("cryptographic key error: {0}")]
    KeyError(String),
}
