//! Role-Based Access Control (RBAC) permission evaluation.

use crate::error::AuthError;
use std::collections::{HashMap, HashSet};

/// Core permission identifiers in the EdgeArena system.
pub mod permissions {
    /// Read access to incoming financial events.
    pub const EVENTS_READ: &str = "events:read";
    /// Submission and ingestion of financial events.
    pub const EVENTS_WRITE: &str = "events:write";
    /// Read access to decision evaluation logs.
    pub const DECISIONS_READ: &str = "decisions:read";
    /// Read access to fraud and compliance rule sets.
    pub const RULES_READ: &str = "rules:read";
    /// Modification and creation of fraud rules.
    pub const RULES_WRITE: &str = "rules:write";
    /// Read-only access to tamper-evident audit logs.
    pub const AUDIT_READ: &str = "audit:read";
    /// Management of users and credential assignments.
    pub const USERS_MANAGE: &str = "users:manage";
}

/// Evaluator for checking user role permissions against protected endpoints.
#[derive(Debug, Clone)]
pub struct RbacAuthorizer {
    role_permissions: HashMap<String, HashSet<String>>,
}

impl Default for RbacAuthorizer {
    fn default() -> Self {
        Self::with_standard_roles()
    }
}

impl RbacAuthorizer {
    /// Initializes an authorizer with standard EdgeArena financial intelligence roles.
    #[must_use]
    pub fn with_standard_roles() -> Self {
        let mut authorizer = Self {
            role_permissions: HashMap::new(),
        };

        // Admin has all permissions wildcard
        authorizer.register_role(
            "admin",
            vec![
                permissions::EVENTS_READ,
                permissions::EVENTS_WRITE,
                permissions::DECISIONS_READ,
                permissions::RULES_READ,
                permissions::RULES_WRITE,
                permissions::AUDIT_READ,
                permissions::USERS_MANAGE,
                "*",
            ],
        );

        // Analyst can read events, read decisions, read/write rules, read audit
        authorizer.register_role(
            "analyst",
            vec![
                permissions::EVENTS_READ,
                permissions::EVENTS_WRITE,
                permissions::DECISIONS_READ,
                permissions::RULES_READ,
                permissions::RULES_WRITE,
                permissions::AUDIT_READ,
            ],
        );

        // Trader can submit events and view their decisions
        authorizer.register_role(
            "trader",
            vec![permissions::EVENTS_WRITE, permissions::DECISIONS_READ],
        );

        // Auditor has read-only access across audit, decisions, and rules
        authorizer.register_role(
            "auditor",
            vec![
                permissions::AUDIT_READ,
                permissions::DECISIONS_READ,
                permissions::RULES_READ,
            ],
        );

        authorizer
    }

    /// Registers a role with associated permission tokens.
    pub fn register_role(&mut self, role: &str, perms: Vec<&str>) {
        let set = self
            .role_permissions
            .entry(role.to_lowercase())
            .or_default();
        for p in perms {
            set.insert(p.to_string());
        }
    }

    /// Checks if any of the provided user roles grant the required permission.
    pub fn authorize(
        &self,
        roles: &[impl AsRef<str>],
        required_permission: &str,
    ) -> Result<(), AuthError> {
        for role in roles {
            let role_name = role.as_ref().to_lowercase();
            if let Some(perms) = self.role_permissions.get(&role_name) {
                if perms.contains("*") || perms.contains(required_permission) {
                    return Ok(());
                }
            }
        }

        Err(AuthError::Forbidden(format!(
            "lacks required permission: '{required_permission}'"
        )))
    }
}
