//! Argon2id password hashing and constant-time verification.

use crate::error::AuthError;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;

/// Hashes a plaintext password using Argon2id with a cryptographically secure random salt.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::PasswordHashing(e.to_string()))?
        .to_string();

    Ok(password_hash)
}

/// Verifies a plaintext password against an Argon2id formatted password hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AuthError::PasswordHashing(format!("invalid hash format: {e}")))?;

    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AuthError::PasswordHashing(e.to_string())),
    }
}
