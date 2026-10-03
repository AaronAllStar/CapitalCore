//! HTTP route handlers for health, metrics, and documentation.

use crate::openapi::generate_openapi_spec;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use edge_observability::{HealthRegistry, HealthStatus, MetricsRegistry};
use serde_json::json;
use std::sync::Arc;

/// Shared application state for foundational endpoints.
#[derive(Clone)]
pub struct AppState {
    /// Registry of subsystem health checks.
    pub health_registry: Arc<HealthRegistry>,
    /// Thread-safe metrics registry.
    pub metrics_registry: Arc<MetricsRegistry>,
}

/// Liveness probe handler returning HTTP 200 OK.
pub async fn liveness_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "UP" })))
}

/// Readiness probe handler assessing all registered subsystems.
pub async fn readiness_handler(State(state): State<AppState>) -> impl IntoResponse {
    let health = state.health_registry.evaluate().await;
    let status_code = match health.status {
        HealthStatus::Healthy | HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status_code, Json(health))
}

/// Metrics scrape handler returning snapshot of all active counters and gauges.
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.metrics_registry.snapshot();
    (StatusCode::OK, Json(snapshot))
}

/// OpenAPI JSON specification handler.
pub async fn openapi_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(generate_openapi_spec()))
}
