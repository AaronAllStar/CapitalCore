//! Persistent database record definitions for users, roles, and permissions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Persistent user account representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserRecord {
    /// Unique user identifier.
    pub id: Uuid,
    /// Unique email address.
    pub email: String,
    /// Salted password hash string (Argon2id).
    pub password_hash: String,
    /// Whether the user account is active.
    pub is_active: bool,
    /// UTC timestamp of creation.
    pub created_at: DateTime<Utc>,
    /// UTC timestamp of last update.
    pub updated_at: DateTime<Utc>,
}

impl UserRecord {
    /// Constructs a new `UserRecord`.
    #[must_use]
    pub fn new(id: Uuid, email: String, password_hash: String, is_active: bool) -> Self {
        let now = Utc::now();
        Self {
            id,
            email,
            password_hash,
            is_active,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Persistent RBAC role representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleRecord {
    /// Unique role identifier.
    pub id: Uuid,
    /// Unique role name (e.g., "admin", "analyst").
    pub name: String,
    /// Optional human-readable description of role scope.
    pub description: Option<String>,
    /// UTC timestamp of creation.
    pub created_at: DateTime<Utc>,
}

/// Persistent RBAC permission representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionRecord {
    /// Unique permission identifier.
    pub id: Uuid,
    /// Unique permission name (e.g., "rules:write", "transactions:review").
    pub name: String,
    /// Optional description of capability granted.
    pub description: Option<String>,
}
