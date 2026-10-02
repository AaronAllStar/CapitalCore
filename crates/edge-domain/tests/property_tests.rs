use chrono::{TimeZone, Utc};
use edge_core::{Currency, Money};
use edge_domain::{
    AuditEvent, AuditId, Channel, Decision, EventId, EventType, FinancialEvent, RiskLevel,
    TransactionId, UserId,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

prop_compose! {
    fn arb_channel()(idx in 0..6_u8) -> Channel {
        match idx {
            0 => Channel::Web,
            1 => Channel::Mobile,
            2 => Channel::Api,
            3 => Channel::Pos,
            4 => Channel::Atm,
            _ => Channel::Batch,
        }
    }
}

prop_compose! {
    fn arb_event_type()(idx in 0..7_u8) -> EventType {
        match idx {
            0 => EventType::Payment,
            1 => EventType::Transfer,
            2 => EventType::Withdrawal,
            3 => EventType::Deposit,
            4 => EventType::Refund,
            5 => EventType::Chargeback,
            _ => EventType::Authorization,
        }
    }
}

prop_compose! {
    fn arb_decision()(idx in 0..4_u8) -> Decision {
        match idx {
            0 => Decision::Allow,
            1 => Decision::Review,
            2 => Decision::Block,
            _ => Decision::Escalate,
        }
    }
}

prop_compose! {
    fn arb_risk_level()(idx in 0..4_u8) -> RiskLevel {
        match idx {
            0 => RiskLevel::Low,
            1 => RiskLevel::Medium,
            2 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }
}

proptest! {
    #[test]
    fn prop_decision_serde_roundtrip(d in arb_decision()) {
        let json = serde_json::to_string(&d).expect("serialize decision");
        let deserialized: Decision = serde_json::from_str(&json).expect("deserialize decision");
        prop_assert_eq!(d, deserialized);
    }

    #[test]
    fn prop_channel_serde_roundtrip(c in arb_channel()) {
        let json = serde_json::to_string(&c).expect("serialize channel");
        let deserialized: Channel = serde_json::from_str(&json).expect("deserialize channel");
        prop_assert_eq!(c, deserialized);
    }

    #[test]
    fn prop_event_type_serde_roundtrip(et in arb_event_type()) {
        let json = serde_json::to_string(&et).expect("serialize event_type");
        let deserialized: EventType = serde_json::from_str(&json).expect("deserialize event_type");
        prop_assert_eq!(et, deserialized);
    }

    #[test]
    fn prop_financial_event_serde_roundtrip(
        amount in 0_i64..100_000_000_i64,
        channel in arb_channel(),
        event_type in arb_event_type(),
        timestamp_sec in 1_600_000_000_i64..1_800_000_000_i64,
    ) {
        let created_at = Utc.timestamp_opt(timestamp_sec, 0).unwrap();
        let mut metadata = BTreeMap::new();
        metadata.insert("source".to_string(), "automated_test".to_string());

        let event = FinancialEvent::new(
            EventId::new(),
            TransactionId::new(),
            UserId::new(),
            event_type,
            channel,
            Money::new(amount, Currency::USD),
            created_at,
            metadata,
        );

        let json = serde_json::to_string(&event).expect("serialize FinancialEvent");
        let deserialized: FinancialEvent = serde_json::from_str(&json).expect("deserialize FinancialEvent");

        prop_assert_eq!(event, deserialized);
    }

    #[test]
    fn prop_audit_event_serde_roundtrip(
        decision in arb_decision(),
        timestamp_sec in 1_600_000_000_i64..1_800_000_000_i64,
    ) {
        let created_at = Utc.timestamp_opt(timestamp_sec, 0).unwrap();
        let mut feature_versions = BTreeMap::new();
        feature_versions.insert("f1".to_string(), "v1".to_string());
        let mut rule_versions = BTreeMap::new();
        rule_versions.insert("r1".to_string(), "v1".to_string());

        let audit = AuditEvent::new(
            AuditId::new(),
            EventId::new(),
            decision,
            vec!["Rule violation".to_string()],
            feature_versions,
            rule_versions,
            Some("model-v1".to_string()),
            created_at,
        );

        let json = serde_json::to_string(&audit).expect("serialize AuditEvent");
        let deserialized: AuditEvent = serde_json::from_str(&json).expect("deserialize AuditEvent");

        prop_assert_eq!(audit, deserialized);
    }
}
