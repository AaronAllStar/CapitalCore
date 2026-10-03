//! Ed25519 (EdDSA) asymmetric JWT signing and verification.

use crate::error::AuthError;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Token purpose / type designation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    /// Short-lived access token for API authorization.
    Access,
    /// Longer-lived token used solely for refreshing credentials.
    Refresh,
}

/// Standardized EdgeArena JWT claims payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: User unique identifier.
    pub sub: Uuid,
    /// Expiration time in seconds since Unix epoch.
    pub exp: i64,
    /// Issued-at time in seconds since Unix epoch.
    pub iat: i64,
    /// Issuer string identifier.
    pub iss: String,
    /// Purpose of the token.
    pub token_type: TokenType,
    /// Assigned RBAC roles.
    pub roles: Vec<String>,
}

impl Claims {
    /// Constructs a new access token claims set with the specified lifetime.
    #[must_use]
    pub fn new_access(sub: Uuid, roles: Vec<String>, ttl_seconds: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub,
            exp: now + ttl_seconds,
            iat: now,
            iss: "edge-arena".to_string(),
            token_type: TokenType::Access,
            roles,
        }
    }

    /// Constructs a new refresh token claims set with the specified lifetime.
    #[must_use]
    pub fn new_refresh(sub: Uuid, ttl_seconds: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub,
            exp: now + ttl_seconds,
            iat: now,
            iss: "edge-arena".to_string(),
            token_type: TokenType::Refresh,
            roles: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtHeader {
    alg: String,
    typ: String,
}

/// Asymmetric Ed25519 keypair for signing and verifying tokens.
pub struct JwtKeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl JwtKeyPair {
    /// Generates a new random Ed25519 keypair using operating system entropy.
    #[must_use]
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Constructs a keypair from 32 private key seed bytes.
    pub fn from_secret_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Returns the raw 32-byte public verifying key.
    #[must_use]
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Signs the given claims into a standard compact EdDSA JWT.
    pub fn sign(&self, claims: &Claims) -> Result<String, AuthError> {
        let header = JwtHeader {
            alg: "EdDSA".to_string(),
            typ: "JWT".to_string(),
        };

        let header_json = serde_json::to_vec(&header)
            .map_err(|e| AuthError::KeyError(format!("header serialization: {e}")))?;
        let claims_json = serde_json::to_vec(claims)
            .map_err(|e| AuthError::KeyError(format!("claims serialization: {e}")))?;

        let header_b64 = URL_SAFE_NO_PAD.encode(header_json);
        let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json);
        let signing_input = format!("{header_b64}.{claims_b64}");

        let signature = self.signing_key.sign(signing_input.as_bytes());
        let sig_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        Ok(format!("{signing_input}.{sig_b64}"))
    }

    /// Verifies the token signature, checks algorithm, and asserts expiration.
    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthError::InvalidToken("token must have 3 segments".into()));
        }

        let header_bytes = URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|e| AuthError::InvalidToken(format!("header base64 decode: {e}")))?;
        let header: JwtHeader = serde_json::from_slice(&header_bytes)
            .map_err(|e| AuthError::InvalidToken(format!("header json decode: {e}")))?;

        if header.alg != "EdDSA" {
            return Err(AuthError::InvalidToken(format!(
                "unsupported algorithm '{}', only EdDSA accepted",
                header.alg
            )));
        }

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let sig_bytes = URL_SAFE_NO_PAD
            .decode(parts[2])
            .map_err(|e| AuthError::InvalidToken(format!("signature base64 decode: {e}")))?;

        let sig_array: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| AuthError::InvalidSignature)?;

        let signature = Signature::from_bytes(&sig_array);
        self.verifying_key
            .verify(signing_input.as_bytes(), &signature)
            .map_err(|_| AuthError::InvalidSignature)?;

        let claims_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| AuthError::InvalidToken(format!("claims base64 decode: {e}")))?;
        let claims: Claims = serde_json::from_slice(&claims_bytes)
            .map_err(|e| AuthError::InvalidToken(format!("claims json decode: {e}")))?;

        let now = Utc::now().timestamp();
        if claims.exp < now {
            return Err(AuthError::ExpiredToken);
        }

        Ok(claims)
    }
}
