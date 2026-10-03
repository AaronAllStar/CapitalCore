//! Decision evaluation service coordinating features, rules, and audit logging.

use crate::error::DecisionError;
use chrono::Utc;
use edge_audit::AuditLog;
use edge_domain::{AuditEvent, AuditId, Decision, FinancialEvent};
use edge_features::FeatureEngine;
use edge_rules::{EvaluationContext, RuleEngine};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::arbitration::{ArbitrationStrategy, MostRestrictive};

/// Full decision outcome packet containing decision, reasons, features, and audit trail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionOutcome {
    /// Final platform decision (ALLOW, REVIEW, BLOCK, ESCALATE).
    pub decision: Decision,
    /// Auditable explanation reason codes.
    pub reasons: Vec<String>,
    /// Cryptographically chained, immutable audit event.
    pub audit_event: AuditEvent,
    /// Features calculated at evaluation time.
    pub features_snapshot: BTreeMap<String, i64>,
}

/// Service executing real-time fraud assessment and policy enforcement.
pub struct DecisionService {
    feature_engine: Arc<FeatureEngine>,
    rule_engine: Arc<RuleEngine>,
    audit_log: Arc<dyn AuditLog>,
    arbitration: Arc<dyn ArbitrationStrategy>,
}

impl DecisionService {
    /// Constructs a new `DecisionService` with default `MostRestrictive` arbitration.
    pub fn new(
        feature_engine: Arc<FeatureEngine>,
        rule_engine: Arc<RuleEngine>,
        audit_log: Arc<dyn AuditLog>,
    ) -> Self {
        Self {
            feature_engine,
            rule_engine,
            audit_log,
            arbitration: Arc::new(MostRestrictive),
        }
    }

    /// Sets a custom decision arbitration strategy.
    #[must_use]
    pub fn with_arbitration(mut self, strategy: Arc<dyn ArbitrationStrategy>) -> Self {
        self.arbitration = strategy;
        self
    }

    /// Evaluates a financial event, computing features, resolving rules, and persisting audit records.
    pub async fn evaluate(&self, event: &FinancialEvent) -> Result<DecisionOutcome, DecisionError> {
        // Step 1: Compute real-time trailing features
        let features = self.feature_engine.compute_and_update(event);

        // Step 2: Assemble evaluation context and run deterministic rule engine
        let ctx = EvaluationContext::new(event.clone(), features.clone());
        let eval_result = self.rule_engine.evaluate(&ctx);

        // Step 3: Construct immutable, reconstructable AuditEvent
        let mut feature_versions = BTreeMap::new();
        for key in features.keys() {
            feature_versions.insert(key.clone(), "1.0.0".to_string());
        }

        let mut rule_versions = BTreeMap::new();
        for m in &eval_result.matches {
            rule_versions.insert(m.rule_name.clone(), "1.0.0".to_string());
        }

        let final_decision = self
            .arbitration
            .arbitrate(&eval_result.matches, Decision::Allow);

        let audit = AuditEvent::new(
            AuditId::new(),
            event.event_id(),
            final_decision,
            eval_result.reasons.clone(),
            feature_versions,
            rule_versions,
            Some("edge-rules-v1".to_string()),
            Utc::now(),
        );

        // Step 4: Persist to append-only tamper-evident audit log
        self.audit_log
            .append(audit.clone())
            .await
            .map_err(|e| DecisionError::Audit(e.to_string()))?;

        Ok(DecisionOutcome {
            decision: final_decision,
            reasons: eval_result.reasons,
            audit_event: audit,
            features_snapshot: features,
        })
    }
}
