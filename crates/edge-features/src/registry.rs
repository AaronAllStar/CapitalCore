//! Feature definitions and metadata registry.

use crate::window::WindowDuration;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Mathematical operation applied over the historical time window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    /// Number of matching events.
    Count,
    /// Sum of minor unit amounts.
    Sum,
    /// Maximum minor unit amount observed.
    Max,
    /// Number of unique string values for a metadata attribute.
    DistinctCount,
}

/// Metadata schema defining a real-time calculated feature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureDefinition {
    /// Unique feature identifier (e.g. "user_txn_count_1h").
    pub name: String,
    /// Human-readable description of the metric.
    pub description: String,
    /// Aggregation operation.
    pub aggregation_type: AggregationType,
    /// Retention duration of the sliding window.
    pub window_seconds: i64,
}

/// Registry storing catalog of supported feature metrics.
#[derive(Debug, Clone, Default)]
pub struct FeatureRegistry {
    features: BTreeMap<String, FeatureDefinition>,
}

impl FeatureRegistry {
    /// Constructs a feature registry populated with standard risk and velocity features.
    #[must_use]
    pub fn with_standard_features() -> Self {
        let mut registry = Self {
            features: BTreeMap::new(),
        };

        registry.register(
            "user_txn_count_5m",
            "Transaction count in trailing 5 minutes",
            AggregationType::Count,
            WindowDuration::Minutes(5),
        );

        registry.register(
            "user_txn_sum_5m",
            "Total amount in minor units in trailing 5 minutes",
            AggregationType::Sum,
            WindowDuration::Minutes(5),
        );

        registry.register(
            "user_txn_count_1h",
            "Transaction count in trailing 1 hour",
            AggregationType::Count,
            WindowDuration::Hours(1),
        );

        registry.register(
            "user_txn_sum_1h",
            "Total amount in minor units in trailing 1 hour",
            AggregationType::Sum,
            WindowDuration::Hours(1),
        );

        registry.register(
            "user_txn_max_1h",
            "Maximum transaction amount in trailing 1 hour",
            AggregationType::Max,
            WindowDuration::Hours(1),
        );

        registry.register(
            "user_txn_count_24h",
            "Transaction count in trailing 24 hours",
            AggregationType::Count,
            WindowDuration::Days(1),
        );

        registry.register(
            "user_txn_sum_24h",
            "Total amount in minor units in trailing 24 hours",
            AggregationType::Sum,
            WindowDuration::Days(1),
        );

        registry.register(
            "user_distinct_merchants_24h",
            "Distinct merchant count in trailing 24 hours",
            AggregationType::DistinctCount,
            WindowDuration::Days(1),
        );

        registry
    }

    /// Registers a feature definition into the catalog.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        aggregation_type: AggregationType,
        window: WindowDuration,
    ) {
        let name_str = name.into();
        self.features.insert(
            name_str.clone(),
            FeatureDefinition {
                name: name_str,
                description: description.into(),
                aggregation_type,
                window_seconds: window.to_seconds(),
            },
        );
    }

    /// Retrieves a feature definition by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&FeatureDefinition> {
        self.features.get(name)
    }

    /// Returns a list of all registered feature names.
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.features.keys().cloned().collect()
    }
}
