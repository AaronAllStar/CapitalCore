use chrono::Utc;
use edge_audit::InMemoryAuditLog;
use edge_core::{Currency, Money};
use edge_decision::DecisionService;
use edge_domain::{
    Channel, Decision, EventId, EventType, FinancialEvent, RuleId, TransactionId, UserId,
};
use edge_features::FeatureEngine;
use edge_rules::{Condition, Rule, RuleAction, RuleEngine};
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

proptest! {
    #[test]
    fn prop_decision_evaluation_is_deterministic(
        amount in 1i64..10_000_000i64,
        threshold in 1i64..10_000_000i64,
    ) {
        let features = Arc::new(FeatureEngine::default());
        let mut rule_engine = RuleEngine::new();
        rule_engine.add_rule(Rule::new(
            RuleId::new(),
            "Block Over Limit",
            1,
            Condition::AmountGreaterThan {
                minor_units: threshold,
                currency: Currency::USD,
            },
            RuleAction::YieldDecision(Decision::Block),
            "BLOCK_LIMIT",
        ));

        let audit = Arc::new(InMemoryAuditLog::new());
        let service = DecisionService::new(features, Arc::new(rule_engine), audit);

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

        let rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt.block_on(async { service.evaluate(&event).await }).unwrap();

        if amount > threshold {
            prop_assert_eq!(res.decision, Decision::Block);
        } else {
            prop_assert_eq!(res.decision, Decision::Allow);
        }
    }
}
