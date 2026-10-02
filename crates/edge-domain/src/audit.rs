use crate::decision::Decision;
use crate::id::EventId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Strongly-typed identifier for an audit log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct AuditId(Uuid);

impl AuditId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for AuditId {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable, reconstructable record of an intelligence engine decision.
///
/// Invariants enforced:
/// - Reconstructable decision per AGENTS.md Invariant #4 (event ID, decision, rules, features, reasons).
/// - Append-only: No mutation methods (`&mut self` is strictly forbidden).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    audit_id: AuditId,
    event_id: EventId,
    decision: Decision,
    reasons: Vec<String>,
    feature_versions: BTreeMap<String, String>,
    rule_versions: BTreeMap<String, String>,
    model_version: Option<String>,
    created_at: DateTime<Utc>,
}

impl AuditEvent {
    /// Constructs a new, immutable `AuditEvent`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        audit_id: AuditId,
        event_id: EventId,
        decision: Decision,
        reasons: Vec<String>,
        feature_versions: BTreeMap<String, String>,
        rule_versions: BTreeMap<String, String>,
        model_version: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            audit_id,
            event_id,
            decision,
            reasons,
            feature_versions,
            rule_versions,
            model_version,
            created_at,
        }
    }

    #[must_use]
    pub const fn audit_id(&self) -> AuditId {
        self.audit_id
    }

    #[must_use]
    pub const fn event_id(&self) -> EventId {
        self.event_id
    }

    #[must_use]
    pub const fn decision(&self) -> Decision {
        self.decision
    }

    #[must_use]
    pub fn reasons(&self) -> &[String] {
        &self.reasons
    }

    #[must_use]
    pub const fn feature_versions(&self) -> &BTreeMap<String, String> {
        &self.feature_versions
    }

    #[must_use]
    pub const fn rule_versions(&self) -> &BTreeMap<String, String> {
        &self.rule_versions
    }

    #[must_use]
    pub fn model_version(&self) -> Option<&str> {
        self.model_version.as_deref()
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
