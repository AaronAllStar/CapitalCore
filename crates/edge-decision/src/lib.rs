//! EdgeArena central decision evaluation engine.
//!
//! Orchestrates features extraction, deterministic rule execution,
//! decision arbitration (ALLOW, REVIEW, BLOCK, ESCALATE), and tamper-evident audit log creation.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod service;

pub use error::DecisionError;
pub use service::{DecisionOutcome, DecisionService};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use edge_audit::InMemoryAuditLog;
    use edge_core::{Currency, Money};
    use edge_domain::{
        Channel, Decision, EventId, EventType, FinancialEvent, RuleId, TransactionId, UserId,
    };
    use edge_features::FeatureEngine;
    use edge_rules::{Condition, Operator, Rule, RuleAction, RuleEngine};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn make_test_event(
        user_id: UserId,
        amount: i64,
        time: chrono::DateTime<Utc>,
    ) -> FinancialEvent {
        FinancialEvent::new(
            EventId::new(),
            TransactionId::new(),
            user_id,
            EventType::Payment,
            Channel::Web,
            Money::new(amount, Currency::USD),
            time,
            BTreeMap::new(),
        )
    }

    #[tokio::test]
    async fn test_decision_service_default_allow() {
        let features = Arc::new(FeatureEngine::default());
        let rules = Arc::new(RuleEngine::new());
        let audit = Arc::new(InMemoryAuditLog::new());
        let service = DecisionService::new(features, rules, audit.clone());

        let event = make_test_event(UserId::new(), 1000, Utc::now());
        let outcome = service.evaluate(&event).await.expect("evaluation succeeds");

        assert_eq!(outcome.decision, Decision::Allow);
        assert!(outcome.reasons.is_empty());
        assert_eq!(audit.count(), 1);
        assert_eq!(outcome.audit_event.decision(), Decision::Allow);
    }

    #[tokio::test]
    async fn test_decision_service_rule_trigger_block() {
        let features = Arc::new(FeatureEngine::default());
        let mut rule_engine = RuleEngine::new();
        rule_engine.add_rule(Rule::new(
            RuleId::new(),
            "Block Over 10k",
            1,
            Condition::AmountGreaterThan {
                minor_units: 1_000_000,
                currency: Currency::USD,
            },
            RuleAction::YieldDecision(Decision::Block),
            "BLOCK_LARGE_AMOUNT",
        ));

        let audit = Arc::new(InMemoryAuditLog::new());
        let service = DecisionService::new(features, Arc::new(rule_engine), audit.clone());

        let event = make_test_event(UserId::new(), 2_500_000, Utc::now());
        let outcome = service.evaluate(&event).await.expect("evaluation succeeds");

        assert_eq!(outcome.decision, Decision::Block);
        assert_eq!(outcome.reasons, vec!["BLOCK_LARGE_AMOUNT"]);
        assert_eq!(audit.count(), 1);
        assert_eq!(outcome.audit_event.decision(), Decision::Block);
        assert!(outcome
            .audit_event
            .rule_versions()
            .contains_key("Block Over 10k"));
    }

    #[tokio::test]
    async fn test_decision_service_velocity_triggered_review() {
        let features = Arc::new(FeatureEngine::default());
        let mut rule_engine = RuleEngine::new();
        rule_engine.add_rule(Rule::new(
            RuleId::new(),
            "Velocity Flag",
            1,
            Condition::FeatureThreshold {
                feature: "user_txn_count_5m".to_string(),
                op: Operator::Gte,
                value: 1,
            },
            RuleAction::YieldDecision(Decision::Review),
            "VELOCITY_LIMIT_EXCEEDED",
        ));

        let audit = Arc::new(InMemoryAuditLog::new());
        let service = DecisionService::new(features, Arc::new(rule_engine), audit.clone());

        let user = UserId::new();
        let now = Utc::now();

        // First transaction -> velocity was 0 -> Allow
        let ev1 = make_test_event(user, 1000, now);
        let res1 = service.evaluate(&ev1).await.expect("eval 1");
        assert_eq!(res1.decision, Decision::Allow);

        // Second transaction 1 minute later -> trailing 5m count is now 1 -> Review
        let ev2 = make_test_event(user, 1000, now + Duration::minutes(1));
        let res2 = service.evaluate(&ev2).await.expect("eval 2");
        assert_eq!(res2.decision, Decision::Review);
        assert_eq!(res2.reasons, vec!["VELOCITY_LIMIT_EXCEEDED"]);
        assert_eq!(audit.count(), 2);
    }
}
