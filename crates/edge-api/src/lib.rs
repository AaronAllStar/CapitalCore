//! EdgeArena HTTP API layer.
//!
//! Provides thin Axum routing, health probes, Prometheus/JSON metrics endpoints,
//! and OpenAPI 3.0 specification generation.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod handlers;
pub mod openapi;
pub mod router;

pub use handlers::{
    liveness_handler, metrics_handler, openapi_handler, readiness_handler, AppState,
};
pub use openapi::generate_openapi_spec;
pub use router::build_router;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use edge_observability::{HealthRegistry, MetricsRegistry};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_app_state() -> AppState {
        AppState {
            health_registry: Arc::new(HealthRegistry::new()),
            metrics_registry: Arc::new(MetricsRegistry::new()),
        }
    }

    #[tokio::test]
    async fn test_liveness_endpoint() {
        let app = build_router(test_app_state());
        let req = Request::builder()
            .uri("/health/liveness")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readiness_endpoint() {
        let app = build_router(test_app_state());
        let req = Request::builder()
            .uri("/health/readiness")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let state = test_app_state();
        let counter = state.metrics_registry.counter("api_requests_total");
        counter.add(42);

        let app = build_router(state);
        let req = Request::builder()
            .uri("/metrics")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_openapi_endpoint() {
        let app = build_router(test_app_state());
        let req = Request::builder()
            .uri("/openapi.json")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
