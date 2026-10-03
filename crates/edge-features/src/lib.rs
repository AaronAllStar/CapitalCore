//! EdgeArena real-time stateful feature extraction and sliding window aggregations.
//!
//! Enforces:
//! - Pure integer arithmetic: zero floating-point representation for money or counts.
//! - Deterministic calculation: ordered window traversals and BTreeMap outputs.
//! - Automatic state pruning past maximum retention windows.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod pipeline;
pub mod registry;
pub mod window;

pub use pipeline::FeatureEngine;
pub use registry::{AggregationType, FeatureDefinition, FeatureRegistry};
pub use window::{EventWindow, WindowDuration, WindowEventEntry};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use edge_core::{Currency, Money};
    use edge_domain::{Channel, EventId, EventType, FinancialEvent, TransactionId, UserId};
    use std::collections::BTreeMap;

    fn make_event(
        user_id: UserId,
        amount_minor: i64,
        created_at: chrono::DateTime<Utc>,
        merchant: Option<&str>,
    ) -> FinancialEvent {
        let mut meta = BTreeMap::new();
        if let Some(m) = merchant {
            meta.insert("merchant_id".to_string(), m.to_string());
        }

        FinancialEvent::new(
            EventId::new(),
            TransactionId::new(),
            user_id,
            EventType::Payment,
            Channel::Web,
            Money::new(amount_minor, Currency::USD),
            created_at,
            meta,
        )
    }

    #[test]
    fn test_sliding_window_count_and_sum() {
        let mut window = EventWindow::new();
        let now = 1_000_000;

        window.record(now - 100, 500, Channel::Web, BTreeMap::new());
        window.record(now - 50, 1500, Channel::Web, BTreeMap::new());
        window.record(now - 10, 2000, Channel::Mobile, BTreeMap::new());

        // Count and sum in last 60 seconds (since now - 60 = 999_940)
        assert_eq!(window.count_since(now - 60), 2);
        assert_eq!(window.sum_since(now - 60), 3500);
        assert_eq!(window.max_since(now - 60), 2000);

        // Count and sum across all 3
        assert_eq!(window.count_since(now - 200), 3);
        assert_eq!(window.sum_since(now - 200), 4000);
    }

    #[test]
    fn test_sliding_window_pruning() {
        let mut window = EventWindow::new();
        let now = 10_000;

        window.record(now - 5000, 100, Channel::Web, BTreeMap::new());
        window.record(now - 100, 200, Channel::Web, BTreeMap::new());

        window.prune_before(now - 1000);
        assert_eq!(window.count_since(0), 1);
        assert_eq!(window.sum_since(0), 200);
    }

    #[test]
    fn test_sliding_window_distinct_merchants() {
        let mut window = EventWindow::new();
        let now = 1000;

        let mut m1 = BTreeMap::new();
        m1.insert("merchant_id".to_string(), "merchant_A".to_string());

        let mut m2 = BTreeMap::new();
        m2.insert("merchant_id".to_string(), "merchant_B".to_string());

        let mut m3 = BTreeMap::new();
        m3.insert("merchant_id".to_string(), "merchant_A".to_string());

        window.record(now - 20, 100, Channel::Web, m1);
        window.record(now - 10, 200, Channel::Web, m2);
        window.record(now - 5, 300, Channel::Web, m3);

        assert_eq!(window.distinct_metadata_since(0, "merchant_id"), 2);
    }

    #[test]
    fn test_feature_engine_trailing_velocity_sequence() {
        let engine = FeatureEngine::default();
        let user_id = UserId::new();
        let base_time = Utc::now();

        // Transaction 1
        let ev1 = make_event(user_id, 1000, base_time, Some("merch_1"));
        let f1 = engine.compute_and_update(&ev1);
        // Before ev1, trailing count is 0
        assert_eq!(f1.get("user_txn_count_1h"), Some(&0));
        assert_eq!(f1.get("user_txn_sum_1h"), Some(&0));

        // Transaction 2 (2 minutes later)
        let ev2 = make_event(
            user_id,
            2500,
            base_time + Duration::minutes(2),
            Some("merch_2"),
        );
        let f2 = engine.compute_and_update(&ev2);
        // Trailing 1h now has 1 transaction of 1000
        assert_eq!(f2.get("user_txn_count_1h"), Some(&1));
        assert_eq!(f2.get("user_txn_sum_1h"), Some(&1000));
        assert_eq!(f2.get("user_txn_count_5m"), Some(&1));

        // Transaction 3 (4 minutes later)
        let ev3 = make_event(
            user_id,
            5000,
            base_time + Duration::minutes(4),
            Some("merch_1"),
        );
        let f3 = engine.compute_and_update(&ev3);
        // Trailing 1h now has 2 transactions (1000 + 2500 = 3500)
        assert_eq!(f3.get("user_txn_count_1h"), Some(&2));
        assert_eq!(f3.get("user_txn_sum_1h"), Some(&3500));
        assert_eq!(f3.get("user_txn_max_1h"), Some(&2500));
        assert_eq!(f3.get("user_distinct_merchants_24h"), Some(&2));
    }
}
