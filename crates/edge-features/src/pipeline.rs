//! Real-time feature calculation engine.

use crate::registry::FeatureRegistry;
use crate::window::EventWindow;
use edge_domain::{FinancialEvent, UserId};
use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;

/// Real-time stateful feature extraction engine.
pub struct FeatureEngine {
    registry: FeatureRegistry,
    user_windows: RwLock<HashMap<UserId, EventWindow>>,
}

impl Default for FeatureEngine {
    fn default() -> Self {
        Self::new(FeatureRegistry::with_standard_features())
    }
}

impl FeatureEngine {
    /// Constructs a new feature engine with specified feature registry.
    #[must_use]
    pub fn new(registry: FeatureRegistry) -> Self {
        Self {
            registry,
            user_windows: RwLock::new(HashMap::new()),
        }
    }

    /// Accesses the underlying feature registry catalog.
    #[must_use]
    pub const fn registry(&self) -> &FeatureRegistry {
        &self.registry
    }

    /// Returns a point-in-time snapshot of the historical window for a specific user.
    #[must_use]
    pub fn snapshot_user_window(&self, user_id: UserId) -> Option<EventWindow> {
        let windows = self.user_windows.read().unwrap();
        windows.get(&user_id).cloned()
    }

    /// Resets all in-memory feature window state across all users.
    pub fn reset(&self) {
        let mut windows = self.user_windows.write().unwrap();
        windows.clear();
    }

    /// Computes trailing historical features for an event, then updates the sliding window state.
    pub fn compute_and_update(&self, event: &FinancialEvent) -> BTreeMap<String, i64> {
        let now_secs = event.created_at().timestamp();
        let mut features = BTreeMap::new();

        let mut windows = self.user_windows.write().unwrap();
        let window = windows.entry(event.user_id()).or_default();

        // Prune events older than maximum window duration (24 hours = 86400 secs)
        window.prune_before(now_secs - 86400);

        // Compute 5-minute features
        let cutoff_5m = now_secs - 300;
        features.insert(
            "user_txn_count_5m".to_string(),
            window.count_since(cutoff_5m),
        );
        features.insert("user_txn_sum_5m".to_string(), window.sum_since(cutoff_5m));

        // Compute 1-hour features
        let cutoff_1h = now_secs - 3600;
        features.insert(
            "user_txn_count_1h".to_string(),
            window.count_since(cutoff_1h),
        );
        features.insert("user_txn_sum_1h".to_string(), window.sum_since(cutoff_1h));
        features.insert("user_txn_max_1h".to_string(), window.max_since(cutoff_1h));

        // Compute 24-hour features
        let cutoff_24h = now_secs - 86400;
        features.insert(
            "user_txn_count_24h".to_string(),
            window.count_since(cutoff_24h),
        );
        features.insert("user_txn_sum_24h".to_string(), window.sum_since(cutoff_24h));
        features.insert(
            "user_distinct_merchants_24h".to_string(),
            window.distinct_metadata_since(cutoff_24h, "merchant_id"),
        );

        // Record current event into the historical window
        window.record(
            now_secs,
            event.money().minor_units(),
            event.channel(),
            event.metadata().clone(),
        );

        features
    }
}
