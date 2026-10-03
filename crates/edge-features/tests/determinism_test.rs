use chrono::{Duration, Utc};
use edge_core::{Currency, Money};
use edge_domain::{Channel, EventId, EventType, FinancialEvent, TransactionId, UserId};
use edge_features::FeatureEngine;
use std::collections::BTreeMap;
use std::sync::Arc;

fn make_test_event(user_id: UserId, amount: i64, time: chrono::DateTime<Utc>) -> FinancialEvent {
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

#[test]
fn test_entity_cross_window_isolation() {
    let engine = FeatureEngine::default();
    let user_a = UserId::new();
    let user_b = UserId::new();
    let now = Utc::now();

    // 10 transactions for User A
    for i in 0..10 {
        let ev = make_test_event(user_a, 1000, now + Duration::seconds(i));
        engine.compute_and_update(&ev);
    }

    // First transaction for User B
    let ev_b = make_test_event(user_b, 500, now + Duration::seconds(15));
    let feat_b = engine.compute_and_update(&ev_b);

    // User B's trailing counts MUST be 0 despite User A having 10 transactions
    assert_eq!(feat_b.get("user_txn_count_1h"), Some(&0));
    assert_eq!(feat_b.get("user_txn_sum_1h"), Some(&0));

    // Next transaction for User A should show 10 trailing transactions
    let ev_a_next = make_test_event(user_a, 2000, now + Duration::seconds(20));
    let feat_a = engine.compute_and_update(&ev_a_next);
    assert_eq!(feat_a.get("user_txn_count_1h"), Some(&10));
    assert_eq!(feat_a.get("user_txn_sum_1h"), Some(&10_000));
}

#[tokio::test]
async fn test_concurrent_feature_computation() {
    let engine = Arc::new(FeatureEngine::default());
    let mut handles = Vec::new();

    for _ in 0..10 {
        let eng = engine.clone();
        handles.push(tokio::spawn(async move {
            let user = UserId::new();
            let base = Utc::now();
            for i in 0..20 {
                let ev = make_test_event(user, 100, base + Duration::seconds(i));
                let f = eng.compute_and_update(&ev);
                assert!(f.contains_key("user_txn_count_1h"));
            }
        }));
    }

    for h in handles {
        h.await.expect("task completes successfully");
    }
}

#[test]
fn test_sliding_window_expiration_determinism() {
    let engine = FeatureEngine::default();
    let user = UserId::new();
    let base = Utc::now();

    // Event at t = 0
    let ev0 = make_test_event(user, 1000, base);
    engine.compute_and_update(&ev0);

    // Event at t = 6 minutes (360 seconds later: outside 5m window, inside 1h window)
    let ev1 = make_test_event(user, 2000, base + Duration::seconds(360));
    let f1 = engine.compute_and_update(&ev1);

    // ev0 expired from 5m window, but still in 1h window
    assert_eq!(f1.get("user_txn_count_5m"), Some(&0));
    assert_eq!(f1.get("user_txn_sum_5m"), Some(&0));
    assert_eq!(f1.get("user_txn_count_1h"), Some(&1));
    assert_eq!(f1.get("user_txn_sum_1h"), Some(&1000));

    // Snapshot serialization check
    let snap = engine.snapshot_user_window(user).expect("snapshot exists");
    let json = serde_json::to_string(&snap).expect("serialize snapshot");
    assert!(json.contains("timestamp_secs"));
}
