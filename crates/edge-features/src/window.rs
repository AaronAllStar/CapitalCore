//! In-memory deterministic sliding time windows and integer aggregations.

use edge_domain::Channel;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Predefined time window intervals for historical metric computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WindowDuration {
    /// Duration in seconds.
    Seconds(i64),
    /// Duration in minutes.
    Minutes(i64),
    /// Duration in hours.
    Hours(i64),
    /// Duration in days.
    Days(i64),
}

impl WindowDuration {
    /// Converts the window duration into total seconds.
    #[must_use]
    pub const fn to_seconds(self) -> i64 {
        match self {
            Self::Seconds(s) => s,
            Self::Minutes(m) => m * 60,
            Self::Hours(h) => h * 3600,
            Self::Days(d) => d * 86400,
        }
    }
}

/// Compact representation of a historical transaction event in a sliding window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowEventEntry {
    /// Unix timestamp in seconds.
    pub timestamp_secs: i64,
    /// Transaction monetary amount in minor units.
    pub amount_minor: i64,
    /// Originating channel.
    pub channel: Channel,
    /// Event metadata attributes.
    pub metadata: BTreeMap<String, String>,
}

/// Time-bounded sliding window for a specific entity.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventWindow {
    entries: Vec<WindowEventEntry>,
}

impl EventWindow {
    /// Creates an empty event window.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Records a new event into the historical window.
    pub fn record(
        &mut self,
        timestamp_secs: i64,
        amount_minor: i64,
        channel: Channel,
        metadata: BTreeMap<String, String>,
    ) {
        self.entries.push(WindowEventEntry {
            timestamp_secs,
            amount_minor,
            channel,
            metadata,
        });
    }

    /// Prunes entries that fall outside the oldest retention threshold.
    pub fn prune_before(&mut self, cutoff_secs: i64) {
        self.entries.retain(|e| e.timestamp_secs >= cutoff_secs);
    }

    /// Counts events that occurred at or after `cutoff_secs`.
    #[must_use]
    pub fn count_since(&self, cutoff_secs: i64) -> i64 {
        self.entries
            .iter()
            .filter(|e| e.timestamp_secs >= cutoff_secs)
            .count() as i64
    }

    /// Calculates sum of amounts for events at or after `cutoff_secs`.
    #[must_use]
    pub fn sum_since(&self, cutoff_secs: i64) -> i64 {
        self.entries
            .iter()
            .filter(|e| e.timestamp_secs >= cutoff_secs)
            .map(|e| e.amount_minor)
            .fold(0i64, |acc, val| acc.saturating_add(val))
    }

    /// Calculates maximum amount for events at or after `cutoff_secs`.
    #[must_use]
    pub fn max_since(&self, cutoff_secs: i64) -> i64 {
        self.entries
            .iter()
            .filter(|e| e.timestamp_secs >= cutoff_secs)
            .map(|e| e.amount_minor)
            .max()
            .unwrap_or(0)
    }

    /// Calculates distinct count of values for a specific metadata key.
    #[must_use]
    pub fn distinct_metadata_since(&self, cutoff_secs: i64, key: &str) -> i64 {
        let mut distinct = BTreeSet::new();
        for e in &self.entries {
            if e.timestamp_secs >= cutoff_secs {
                if let Some(val) = e.metadata.get(key) {
                    distinct.insert(val.clone());
                }
            }
        }
        distinct.len() as i64
    }
}
