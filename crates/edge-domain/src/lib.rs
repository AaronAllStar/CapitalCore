//! # `edge-domain`
//!
//! Core domain entities, identifiers, policy decisions, and audit events
//! for the EdgeArena financial intelligence platform.
//!
//! Invariants:
//! - Zero external I/O or database dependencies.
//! - Strongly-typed identifiers wrapping UUIDs.
//! - Canonical decisions restricted to ALLOW, REVIEW, BLOCK, ESCALATE.
//! - Strictly immutable financial and audit events.

#![deny(unsafe_code)]

pub mod audit;
pub mod decision;
pub mod enums;
pub mod error;
pub mod event;
pub mod id;

pub use audit::{AuditEvent, AuditId};
pub use decision::{Decision, DecisionOutcome};
pub use enums::{Channel, EventType, RiskLevel};
pub use error::DomainError;
pub use event::FinancialEvent;
pub use id::{EventId, ModelId, RuleId, TransactionId, UserId};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use edge_core::{Currency, Money};
    use std::collections::BTreeMap;

    #[test]
    fn test_decision_variants_count() {
        // Enforce exactly 4 variants per AGENTS.md
        let variants = [
            Decision::Allow,
            Decision::Review,
            Decision::Block,
            Decision::Escalate,
        ];
        assert_eq!(variants.len(), 4);

        for variant in &variants {
            match variant {
                Decision::Allow => assert!(variant.is_allowed()),
                Decision::Block => assert!(variant.is_blocked()),
                Decision::Review | Decision::Escalate => {
                    assert!(!variant.is_allowed());
                    assert!(!variant.is_blocked());
                }
            }
        }
    }

    #[test]
    fn test_financial_event_serde_roundtrip() {
        let mut metadata = BTreeMap::new();
        metadata.insert("ip".to_string(), "192.168.1.1".to_string());
        metadata.insert("device_id".to_string(), "dev-987".to_string());

        let event = FinancialEvent::new(
            EventId::new(),
            TransactionId::new(),
            UserId::new(),
            EventType::Payment,
            Channel::Web,
            Money::new(4500, Currency::USD),
            Utc::now(),
            metadata,
        );

        let json = serde_json::to_string(&event).expect("serialization");
        let deserialized: FinancialEvent = serde_json::from_str(&json).expect("deserialization");

        assert_eq!(event.event_id(), deserialized.event_id());
        assert_eq!(event.transaction_id(), deserialized.transaction_id());
        assert_eq!(event.user_id(), deserialized.user_id());
        assert_eq!(event.money(), deserialized.money());
        assert_eq!(event.channel(), deserialized.channel());
        assert_eq!(event.event_type(), deserialized.event_type());
        assert_eq!(event.get_metadata("ip"), Some("192.168.1.1"));
    }

    #[test]
    fn test_audit_event_serde_roundtrip() {
        let mut feature_versions = BTreeMap::new();
        feature_versions.insert("velocity_1h".to_string(), "1.2.0".to_string());

        let mut rule_versions = BTreeMap::new();
        rule_versions.insert("high_amount_check".to_string(), "2.0.1".to_string());

        let audit = AuditEvent::new(
            AuditId::new(),
            EventId::new(),
            Decision::Block,
            vec!["Velocity threshold exceeded".to_string()],
            feature_versions,
            rule_versions,
            Some("xgboost-fraud-v3".to_string()),
            Utc::now(),
        );

        let json = serde_json::to_string(&audit).expect("serialization");
        let deserialized: AuditEvent = serde_json::from_str(&json).expect("deserialization");

        assert_eq!(audit.audit_id(), deserialized.audit_id());
        assert_eq!(audit.event_id(), deserialized.event_id());
        assert_eq!(audit.decision(), deserialized.decision());
        assert_eq!(audit.reasons(), deserialized.reasons());
        assert_eq!(audit.model_version(), Some("xgboost-fraud-v3"));
    }
}
