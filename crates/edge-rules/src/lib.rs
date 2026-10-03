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
pub mod evaluator;

pub use ast::{Condition, Operator, Rule, RuleAction};
pub use context::EvaluationContext;
pub use engine::{RuleEngine, RuleEvaluationResult, RuleMatch};
pub use evaluator::evaluate_condition;

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
}
