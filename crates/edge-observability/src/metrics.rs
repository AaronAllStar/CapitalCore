//! In-memory thread-safe metrics counters and gauges.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

/// Monotonically increasing unsigned 64-bit counter.
#[derive(Debug, Default)]
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    /// Creates a new counter starting at zero.
    #[must_use]
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    /// Increments the counter by 1.
    pub fn inc(&self) {
        self.add(1);
    }

    /// Increments the counter by a given amount.
    pub fn add(&self, amount: u64) {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }

    /// Returns the current value.
    #[must_use]
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Signed 64-bit gauge that can increase or decrease.
#[derive(Debug, Default)]
pub struct Gauge {
    value: AtomicI64,
}

impl Gauge {
    /// Creates a new gauge starting at zero.
    #[must_use]
    pub fn new() -> Self {
        Self {
            value: AtomicI64::new(0),
        }
    }

    /// Sets the gauge to an absolute value.
    pub fn set(&self, val: i64) {
        self.value.store(val, Ordering::Relaxed);
    }

    /// Increments the gauge.
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements the gauge.
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Returns the current value.
    #[must_use]
    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Serializable snapshot of current metric values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Named unsigned counter values.
    pub counters: BTreeMap<String, u64>,
    /// Named signed gauge values.
    pub gauges: BTreeMap<String, i64>,
}

/// Registry for creating and recording application metrics.
#[derive(Default, Clone)]
pub struct MetricsRegistry {
    counters: Arc<std::sync::RwLock<BTreeMap<String, Arc<Counter>>>>,
    gauges: Arc<std::sync::RwLock<BTreeMap<String, Arc<Gauge>>>>,
}

impl MetricsRegistry {
    /// Creates a new empty `MetricsRegistry`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            counters: Arc::new(std::sync::RwLock::new(BTreeMap::new())),
            gauges: Arc::new(std::sync::RwLock::new(BTreeMap::new())),
        }
    }

    /// Returns or creates a named counter.
    pub fn counter(&self, name: impl Into<String>) -> Arc<Counter> {
        let name = name.into();
        let mut counters = self.counters.write().unwrap();
        counters
            .entry(name)
            .or_insert_with(|| Arc::new(Counter::new()))
            .clone()
    }

    /// Returns or creates a named gauge.
    pub fn gauge(&self, name: impl Into<String>) -> Arc<Gauge> {
        let name = name.into();
        let mut gauges = self.gauges.write().unwrap();
        gauges
            .entry(name)
            .or_insert_with(|| Arc::new(Gauge::new()))
            .clone()
    }

    /// Produces a point-in-time snapshot of all metric values.
    #[must_use]
    pub fn snapshot(&self) -> MetricsSnapshot {
        let counters = self.counters.read().unwrap();
        let gauges = self.gauges.read().unwrap();

        let mut counter_vals = BTreeMap::new();
        for (name, c) in counters.iter() {
            counter_vals.insert(name.clone(), c.get());
        }

        let mut gauge_vals = BTreeMap::new();
        for (name, g) in gauges.iter() {
            gauge_vals.insert(name.clone(), g.get());
        }

        MetricsSnapshot {
            counters: counter_vals,
            gauges: gauge_vals,
        }
    }
}
