use chrono::Utc;
use edge_core::{Currency, Money};
use edge_domain::{
    Channel, Decision, EventId, EventType, FinancialEvent, RuleId, TransactionId, UserId,
};
use edge_rules::{Condition, EvaluationContext, Rule, RuleAction, RuleEngine};
use proptest::prelude::*;
use std::collections::BTreeMap;

proptest! {
    #[test]
    fn prop_rule_engine_deterministic_decision(
        amount in 1i64..10_000_000i64,
        threshold in 1i64..10_000_000i64,
    ) {
        let mut engine = RuleEngine::new();
        let rule = Rule::new(
            RuleId::new(),
            "Prop Test Rule",
            1,
            Condition::AmountGreaterThan {
                minor_units: threshold,
                currency: Currency::USD,
            },
            RuleAction::YieldDecision(Decision::Block),
            "PROP_BLOCK",
        );
        engine.add_rule(rule);

        let event = FinancialEvent::new(
            EventId::new(),
            TransactionId::new(),
            UserId::new(),
            EventType::Payment,
            Channel::Web,
            Money::new(amount, Currency::USD),
            Utc::now(),
            BTreeMap::new(),
        );

        let ctx = EvaluationContext::for_event(event);
        let res1 = engine.evaluate(&ctx);
        let res2 = engine.evaluate(&ctx);

        // Strict determinism invariant
        prop_assert_eq!(res1.final_decision, res2.final_decision);
        prop_assert_eq!(res1.reasons, res2.reasons);

        if amount > threshold {
            prop_assert_eq!(res1.final_decision, Decision::Block);
        } else {
            prop_assert_eq!(res1.final_decision, Decision::Allow);
        }
    }
}
