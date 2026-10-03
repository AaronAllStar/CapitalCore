//! Auditable explanation generation for regulatory compliance and audit logs.

use crate::engine::{RuleEvaluationResult, RuleMatch};
use edge_domain::Decision;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Structured audit explanation detailing the exact rules and reasons triggering a decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditExplanation {
    /// Final decision outcome.
    pub decision: Decision,
    /// Primary explanation reason code.
    pub primary_reason: Option<String>,
    /// Full list of matched rules and emitted decisions.
    pub rule_matches: Vec<RuleMatch>,
    /// Map of active rule names to semantic versions for deterministic decision reconstruction.
    pub rule_versions: BTreeMap<String, String>,
}

/// Generates an immutable, reconstructable `AuditExplanation` from a `RuleEvaluationResult`.
#[must_use]
pub fn generate_audit_explanation(res: &RuleEvaluationResult) -> AuditExplanation {
    let primary_reason = res.reasons.first().cloned();
    let mut rule_versions = BTreeMap::new();

    for m in &res.matches {
        rule_versions.insert(m.rule_name.clone(), "1.0.0".to_string());
    }

    AuditExplanation {
        decision: res.final_decision,
        primary_reason,
        rule_matches: res.matches.clone(),
        rule_versions,
    }
}
