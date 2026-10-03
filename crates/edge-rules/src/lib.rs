//! EdgeArena deterministic rule evaluation engine, AST predicates, and decision resolution.
//!
//! Enforces:
//! - Pure functions with zero side-effects during evaluation.
//! - Deterministic ordering by priority and identifier.
//! - Decision conflict resolution via explicit severity ranking.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod ast;
pub mod context;
pub mod engine;
pub mod error;
pub mod evaluator;
pub mod explanation;
pub mod repository;

pub use ast::{Condition, Operator, Rule, RuleAction};
pub use context::EvaluationContext;
pub use engine::{RuleEngine, RuleEvaluationResult, RuleMatch};
pub use error::RuleError;
pub use evaluator::evaluate_condition;
pub use explanation::{generate_audit_explanation, AuditExplanation};
pub use repository::{InMemoryRuleRepository, RuleRepository};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use edge_core::{Currency, Money};
    use edge_domain::{
        Channel, Decision, EventId, EventType, FinancialEvent, RuleId, TransactionId, UserId,
    };
    use std::collections::BTreeMap;

    fn dummy_event(
        amount_minor: i64,
        currency: Currency,
        channel: Channel,
        event_type: EventType,
    ) -> FinancialEvent {
        FinancialEvent::new(
            EventId::new(),
            TransactionId::new(),
            UserId::new(),
            event_type,
            channel,
            Money::new(amount_minor, currency),
            Utc::now(),
            BTreeMap::new(),
        )
    }

    #[test]
    fn test_rule_engine_empty_returns_allow() {
        let engine = RuleEngine::new();
        let event = dummy_event(1000, Currency::USD, Channel::Web, EventType::Payment);
        let ctx = EvaluationContext::for_event(event);

        let res = engine.evaluate(&ctx);
        assert_eq!(res.final_decision, Decision::Allow);
        assert!(res.matches.is_empty());
    }

    #[test]
    fn test_rule_engine_amount_block() {
        let mut engine = RuleEngine::new();
        let rule = Rule::new(
            RuleId::new(),
            "Block High Value Transfers",
            1,
            Condition::And(vec![
                Condition::EventTypeEquals(EventType::Transfer),
                Condition::AmountGreaterThan {
                    minor_units: 1_000_000,
                    currency: Currency::USD,
                },
            ]),
            RuleAction::YieldDecision(Decision::Block),
            "HIGH_VALUE_TRANSFER_BLOCKED",
        );
        engine.add_rule(rule);

        // Test below threshold
        let small_ev = dummy_event(500_000, Currency::USD, Channel::Web, EventType::Transfer);
        let ctx_small = EvaluationContext::for_event(small_ev);
        assert_eq!(engine.evaluate(&ctx_small).final_decision, Decision::Allow);

        // Test above threshold
        let large_ev = dummy_event(2_000_000, Currency::USD, Channel::Web, EventType::Transfer);
        let ctx_large = EvaluationContext::for_event(large_ev);
        let res = engine.evaluate(&ctx_large);
        assert_eq!(res.final_decision, Decision::Block);
        assert_eq!(res.reasons, vec!["HIGH_VALUE_TRANSFER_BLOCKED"]);
    }

    #[test]
    fn test_rule_precedence_block_over_review() {
        let mut engine = RuleEngine::new();

        let review_rule = Rule::new(
            RuleId::new(),
            "Review Web Payments",
            10,
            Condition::ChannelEquals(Channel::Web),
            RuleAction::YieldDecision(Decision::Review),
            "WEB_PAYMENT_FLAG",
        );

        let block_rule = Rule::new(
            RuleId::new(),
            "Block Large Payments",
            20,
            Condition::AmountGreaterThan {
                minor_units: 50_000,
                currency: Currency::USD,
            },
            RuleAction::YieldDecision(Decision::Block),
            "LARGE_PAYMENT_BLOCK",
        );

        engine.add_rule(review_rule);
        engine.add_rule(block_rule);

        let event = dummy_event(100_000, Currency::USD, Channel::Web, EventType::Payment);
        let ctx = EvaluationContext::for_event(event);

        let res = engine.evaluate(&ctx);
        // Both match, but Block has higher severity than Review
        assert_eq!(res.final_decision, Decision::Block);
        assert_eq!(res.matches.len(), 2);
        assert!(res.reasons.contains(&"WEB_PAYMENT_FLAG".to_string()));
        assert!(res.reasons.contains(&"LARGE_PAYMENT_BLOCK".to_string()));
    }

    #[test]
    fn test_rule_feature_threshold() {
        let mut engine = RuleEngine::new();
        let rule = Rule::new(
            RuleId::new(),
            "High Velocity Review",
            5,
            Condition::FeatureThreshold {
                feature: "user_txn_velocity_1h".to_string(),
                op: Operator::Gte,
                value: 5,
            },
            RuleAction::YieldDecision(Decision::Review),
            "VELOCITY_EXCEEDED",
        );
        engine.add_rule(rule);

        let event = dummy_event(1000, Currency::USD, Channel::Mobile, EventType::Payment);
        let mut features = BTreeMap::new();
        features.insert("user_txn_velocity_1h".to_string(), 6);

        let ctx = EvaluationContext::new(event, features);
        let res = engine.evaluate(&ctx);
        assert_eq!(res.final_decision, Decision::Review);
        assert_eq!(res.reasons, vec!["VELOCITY_EXCEEDED"]);
    }

    #[tokio::test]
    async fn test_rule_repository_crud() {
        let repo = InMemoryRuleRepository::new();
        let rule_id = RuleId::new();
        let rule = Rule::new(
            rule_id,
            "ATM Withdrawal Limit",
            1,
            Condition::ChannelEquals(Channel::Atm),
            RuleAction::YieldDecision(Decision::Review),
            "ATM_RULE",
        );

        repo.save(rule.clone()).await.expect("save rule");

        let fetched = repo
            .get_by_id(rule_id)
            .await
            .expect("fetch rule")
            .expect("exists");
        assert_eq!(fetched.name, "ATM Withdrawal Limit");

        let all = repo.get_all().await.expect("get all rules");
        assert_eq!(all.len(), 1);

        let deleted = repo.delete(rule_id).await.expect("delete rule");
        assert!(deleted);

        let missing = repo.get_by_id(rule_id).await.expect("fetch after delete");
        assert!(missing.is_none());
    }

    #[test]
    fn test_audit_explanation_generation() {
        let rule_id = RuleId::new();
        let eval_result = RuleEvaluationResult {
            final_decision: Decision::Block,
            matches: vec![RuleMatch {
                rule_id,
                rule_name: "Block Rule".to_string(),
                decision: Decision::Block,
                reason_code: "BLOCK_REASON".to_string(),
            }],
            reasons: vec!["BLOCK_REASON".to_string()],
        };

        let explanation = generate_audit_explanation(&eval_result);
        assert_eq!(explanation.decision, Decision::Block);
        assert_eq!(explanation.primary_reason, Some("BLOCK_REASON".to_string()));
        assert_eq!(explanation.rule_matches.len(), 1);
        assert_eq!(
            explanation.rule_versions.get("Block Rule"),
            Some(&"1.0.0".to_string())
        );
    }
}
