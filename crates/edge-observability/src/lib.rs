//! EdgeArena observability, structured logging, metrics, and health checking.
//!
//! Provides JSON structured logging, extensible asynchronous subsystem health probes,
//! and thread-safe metrics counters/gauges.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod health;
pub mod logging;
pub mod metrics;

pub use health::{
    BoxFuture, ComponentHealth, HealthCheck, HealthRegistry, HealthStatus, SystemHealth,
};
pub use logging::{LogFormat, ObservabilityConfig, TracingInitError};
pub use metrics::{Counter, Gauge, MetricsRegistry, MetricsSnapshot};

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyCheck {
        name: &'static str,
        status: HealthStatus,
    }

    impl HealthCheck for DummyCheck {
        fn name(&self) -> &str {
            self.name
        }

        fn check<'a>(&'a self) -> BoxFuture<'a, ComponentHealth> {
            Box::pin(async move {
                match self.status {
                    HealthStatus::Healthy => ComponentHealth::healthy(self.name, 1),
                    HealthStatus::Degraded => {
                        ComponentHealth::degraded(self.name, "high latency", 250)
                    }
                    HealthStatus::Unhealthy => {
                        ComponentHealth::unhealthy(self.name, "connection refused", 500)
                    }
                }
            })
        }
    }

    #[tokio::test]
    async fn test_health_aggregation_all_healthy() {
        let mut registry = HealthRegistry::new();
        registry.register(Box::new(DummyCheck {
            name: "db",
            status: HealthStatus::Healthy,
        }));
        registry.register(Box::new(DummyCheck {
            name: "cache",
            status: HealthStatus::Healthy,
        }));

        let eval = registry.evaluate().await;
        assert_eq!(eval.status, HealthStatus::Healthy);
        assert_eq!(eval.components.len(), 2);
    }

    #[tokio::test]
    async fn test_health_aggregation_one_degraded() {
        let mut registry = HealthRegistry::new();
        registry.register(Box::new(DummyCheck {
            name: "db",
            status: HealthStatus::Healthy,
        }));
        registry.register(Box::new(DummyCheck {
            name: "queue",
            status: HealthStatus::Degraded,
        }));

        let eval = registry.evaluate().await;
        assert_eq!(eval.status, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_health_aggregation_one_unhealthy() {
        let mut registry = HealthRegistry::new();
        registry.register(Box::new(DummyCheck {
            name: "db",
            status: HealthStatus::Unhealthy,
        }));
        registry.register(Box::new(DummyCheck {
            name: "cache",
            status: HealthStatus::Degraded,
        }));

        let eval = registry.evaluate().await;
        assert_eq!(eval.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_metrics_counter_and_gauge() {
        let registry = MetricsRegistry::new();
        let events_counter = registry.counter("events_processed_total");
        events_counter.inc();
        events_counter.add(9);
        assert_eq!(events_counter.get(), 10);

        let active_tasks = registry.gauge("active_tasks");
        active_tasks.inc();
        active_tasks.inc();
        active_tasks.dec();
        assert_eq!(active_tasks.get(), 1);

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.counters.get("events_processed_total"), Some(&10));
        assert_eq!(snapshot.gauges.get("active_tasks"), Some(&1));

        let json = serde_json::to_string(&snapshot).expect("serialize metrics");
        assert!(json.contains("events_processed_total"));
    }

    #[test]
    fn test_observability_config_creation() {
        let config = ObservabilityConfig::new("warn,edge_arena=debug", LogFormat::Json);
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.env_filter, "warn,edge_arena=debug");
    }
}
