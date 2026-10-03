//! Authentication extractors and authorization middleware helpers.

use edge_auth::{AuthError, JwtKeyPair, RbacAuthorizer};
use edge_security::ErrorResponse;
use uuid::Uuid;

/// Identity and role attributes extracted from a verified JWT token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedUser {
    /// Subject user identifier.
    pub user_id: Uuid,
    /// Roles assigned to the subject.
    pub roles: Vec<String>,
}

impl AuthenticatedUser {
    /// Constructs a new `AuthenticatedUser`.
    #[must_use]
    pub fn new(user_id: Uuid, roles: Vec<String>) -> Self {
        Self { user_id, roles }
    }

    /// Verifies if user possesses the required RBAC permission.
    pub fn authorize(
        &self,
        authorizer: &RbacAuthorizer,
        permission: &str,
    ) -> Result<(), ErrorResponse> {
        authorizer
            .authorize(&self.roles, permission)
            .map_err(|e| ErrorResponse::forbidden(e.to_string()))
    }
}

/// Extracts and verifies the bearer token from the HTTP Authorization header.
pub fn authenticate_header(
    auth_header: Option<&str>,
    keypair: &JwtKeyPair,
) -> Result<AuthenticatedUser, ErrorResponse> {
    let header_val =
        auth_header.ok_or_else(|| ErrorResponse::unauthorized("missing Authorization header"))?;

    let token = header_val
        .strip_prefix("Bearer ")
        .or_else(|| header_val.strip_prefix("bearer "))
        .ok_or_else(|| {
            ErrorResponse::unauthorized("invalid Authorization format, expected 'Bearer <token>'")
        })?;

    let claims = keypair.verify(token).map_err(|err| match err {
        AuthError::ExpiredToken => ErrorResponse::unauthorized("token has expired"),
        AuthError::InvalidSignature => ErrorResponse::unauthorized("invalid token signature"),
        other => ErrorResponse::unauthorized(other.to_string()),
    })?;

    Ok(AuthenticatedUser::new(claims.sub, claims.roles))
}
