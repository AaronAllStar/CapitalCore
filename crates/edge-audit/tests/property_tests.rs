use chrono::{TimeZone, Utc};
use edge_audit::{AuditLog, InMemoryAuditLog};
use edge_domain::{AuditEvent, AuditId, Decision, EventId};
use proptest::prelude::*;
use std::collections::BTreeMap;

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

proptest! {
    /// Property: Any valid appended event is always retrievable by its AuditId and EventId.
    #[test]
    fn prop_appended_event_always_retrievable(
        decision in arb_decision(),
        timestamp_sec in 1_600_000_000_i64..1_800_000_000_i64,
        reason in "[a-zA-Z0-9 ]{1,30}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let log = InMemoryAuditLog::new();
            let event_id = EventId::new();
            let audit_id = AuditId::new();
            let created_at = Utc.timestamp_opt(timestamp_sec, 0).unwrap();

            let mut features = BTreeMap::new();
            features.insert("f".to_string(), "1".to_string());
            let mut rules = BTreeMap::new();
            rules.insert("r".to_string(), "1".to_string());

            let event = AuditEvent::new(
                audit_id,
                event_id,
                decision,
                vec![reason],
                features,
                rules,
                None,
                created_at,
            );

            let appended_id = log.append(event.clone()).await.expect("append");
            assert_eq!(appended_id, audit_id);

            // Retrieve by AuditId
            let retrieved = log.get_by_audit_id(audit_id).await.expect("query").expect("found");
            assert_eq!(retrieved, event);

            // Retrieve by EventId
            let by_event = log.get_by_event_id(event_id).await.expect("query");
            assert_eq!(by_event.len(), 1);
            assert_eq!(by_event[0], event);

            // Verify integrity
            let valid = log.verify_integrity().await.expect("integrity check");
            assert!(valid);
        });
    }
}
