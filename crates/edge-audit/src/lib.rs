//! # `edge-audit`
//!
//! Append-only, tamper-evident audit logging for the EdgeArena intelligence platform.
//!
//! Invariants strictly enforced:
//! - Strictly append-only: zero update or delete methods exist on `AuditLog`.
//! - Decision reconstructability: every outcome links to inputs, features, rules, and model versions.
//! - Tamper evidence: sequenced hash chaining validating audit history integrity.

#![deny(unsafe_code)]

pub mod error;
pub mod in_memory;
pub mod log;
pub mod record;

pub use error::AuditError;
pub use in_memory::InMemoryAuditLog;
pub use log::{AuditLog, BoxFuture};
pub use record::AuditRecord;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use edge_domain::{AuditEvent, AuditId, Decision, EventId};
    use std::collections::BTreeMap;

    fn create_test_event(event_id: EventId, decision: Decision) -> AuditEvent {
        let mut feature_versions = BTreeMap::new();
        feature_versions.insert("txn_freq_24h".to_string(), "1.0.0".to_string());

        let mut rule_versions = BTreeMap::new();
        rule_versions.insert("block_high_velocity".to_string(), "1.0.0".to_string());

        AuditEvent::new(
            AuditId::new(),
            event_id,
            decision,
            vec!["Rule match".to_string()],
            feature_versions,
            rule_versions,
            Some("fraud-detector-v1".to_string()),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn test_append_and_query_by_audit_id() {
        let log = InMemoryAuditLog::new();
        let event_id = EventId::new();
        let event = create_test_event(event_id, Decision::Block);
        let audit_id = event.audit_id();

        let appended_id = log.append(event.clone()).await.expect("append");
        assert_eq!(appended_id, audit_id);
        assert_eq!(log.count(), 1);

        let retrieved = log
            .get_by_audit_id(audit_id)
            .await
            .expect("query")
            .expect("found");
        assert_eq!(retrieved.audit_id(), audit_id);
        assert_eq!(retrieved.decision(), Decision::Block);
    }

    #[tokio::test]
    async fn test_query_by_event_id() {
        let log = InMemoryAuditLog::new();
        let target_event_id = EventId::new();
        let other_event_id = EventId::new();

        let e1 = create_test_event(target_event_id, Decision::Review);
        let e2 = create_test_event(target_event_id, Decision::Escalate);
        let e3 = create_test_event(other_event_id, Decision::Allow);

        log.append(e1).await.expect("append");
        log.append(e2).await.expect("append");
        log.append(e3).await.expect("append");

        let target_events = log.get_by_event_id(target_event_id).await.expect("query");
        assert_eq!(target_events.len(), 2);
        assert_eq!(target_events[0].decision(), Decision::Review);
        assert_eq!(target_events[1].decision(), Decision::Escalate);
    }

    #[tokio::test]
    async fn test_query_time_range() {
        let log = InMemoryAuditLog::new();
        let now = Utc::now();

        let past = now - Duration::hours(2);
        let future = now + Duration::hours(2);

        let e1 = AuditEvent::new(
            AuditId::new(),
            EventId::new(),
            Decision::Allow,
            vec![],
            BTreeMap::new(),
            BTreeMap::new(),
            None,
            now - Duration::hours(3), // Outside range
        );

        let e2 = AuditEvent::new(
            AuditId::new(),
            EventId::new(),
            Decision::Block,
            vec![],
            BTreeMap::new(),
            BTreeMap::new(),
            None,
            now, // Inside range
        );

        log.append(e1).await.expect("append");
        log.append(e2).await.expect("append");

        let in_range = log.query_range(past, future).await.expect("range query");
        assert_eq!(in_range.len(), 1);
        assert_eq!(in_range[0].decision(), Decision::Block);
    }

    #[tokio::test]
    async fn test_tamper_evident_hash_chain() {
        let log = InMemoryAuditLog::new();

        for i in 0..5 {
            let decision = if i % 2 == 0 {
                Decision::Allow
            } else {
                Decision::Block
            };
            let e = create_test_event(EventId::new(), decision);
            log.append(e).await.expect("append");
        }

        assert_eq!(log.count(), 5);
        let is_valid = log.verify_integrity().await.expect("verify");
        assert!(is_valid);
    }
}
