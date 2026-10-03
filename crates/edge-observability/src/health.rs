//! Health check infrastructure and status aggregation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

/// Operational status of a component or system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Fully operational and meeting SLAs.
    Healthy = 0,
    /// Operating under degraded performance or partial connectivity.
    Degraded = 1,
    /// Non-functional or failed core dependency.
    Unhealthy = 2,
}

/// Detailed health check result for a single subsystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Name of the monitored component (e.g., "postgres", "event_bus").
    pub name: String,
    /// Reported operational status.
    pub status: HealthStatus,
    /// Optional diagnostic details or error message.
    pub details: Option<String>,
    /// Execution duration of the health probe in milliseconds.
    pub latency_ms: u64,
}

impl ComponentHealth {
    /// Constructs a healthy component report.
    #[must_use]
    pub fn healthy(name: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            details: None,
            latency_ms,
        }
    }

    /// Constructs a degraded component report.
    #[must_use]
    pub fn degraded(name: impl Into<String>, details: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            details: Some(details.into()),
            latency_ms,
        }
    }

    /// Constructs an unhealthy component report.
    #[must_use]
    pub fn unhealthy(name: impl Into<String>, details: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            details: Some(details.into()),
            latency_ms,
        }
    }
}

/// Aggregated system-wide health status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall health summary.
    pub status: HealthStatus,
    /// Component breakdown reports.
    pub components: Vec<ComponentHealth>,
    /// UTC timestamp of evaluation.
    pub timestamp: DateTime<Utc>,
}

/// Pinned future for asynchronous health probes.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait implemented by components that provide liveness/readiness probes.
pub trait HealthCheck: Send + Sync {
    /// Returns the unique name of this health probe.
    fn name(&self) -> &str;

    /// Executes the health check asynchronously.
    fn check<'a>(&'a self) -> BoxFuture<'a, ComponentHealth>;
}

/// Registry and aggregator for registered subsystem health checks.
#[derive(Default)]
pub struct HealthRegistry {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl HealthRegistry {
    /// Constructs an empty `HealthRegistry`.
    #[must_use]
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    /// Registers a new component health check.
    pub fn register(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }

    /// Evaluates all registered health checks and computes aggregated status.
    pub async fn evaluate(&self) -> SystemHealth {
        let mut components = Vec::with_capacity(self.checks.len());
        let mut overall_status = HealthStatus::Healthy;

        for check in &self.checks {
            let result = check.check().await;
            if result.status > overall_status {
                overall_status = result.status;
            }
            components.push(result);
        }

        SystemHealth {
            status: overall_status,
            components,
            timestamp: Utc::now(),
        }
    }
}
