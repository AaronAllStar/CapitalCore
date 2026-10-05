//! EdgeArena HTTP API layer.
//!
//! Provides thin Axum routing, health probes, Prometheus/JSON metrics endpoints,
//! authentication and authorization middleware, and OpenAPI 3.0 specification generation.

#![deny(unsafe_code)]
#![warn(missing_docs)]

/// Authentication and JWT extractors.
pub mod auth;
/// Live in-memory decision store.
pub mod decision_store;
/// HTTP route handlers.
pub mod handlers;
/// OpenAPI 3.0 specification.
pub mod openapi;
/// Axum router assembly.
pub mod router;
/// Transaction submission route.
pub mod transaction;

pub use auth::{authenticate_header, AuthenticatedUser};
pub use decision_store::{DecisionStats, RecentDecisions, StoredDecision};
pub use handlers::{
    get_audit_records_handler, get_decision_by_id_handler, get_decision_stats_handler,
    get_decisions_handler, get_demo_token_handler, get_model_info_handler, liveness_handler,
    metrics_handler, openapi_handler, readiness_handler, verify_audit_integrity_handler, AppState,
};
pub use openapi::generate_openapi_spec;
pub use router::build_router;
pub use transaction::submit_transaction_handler;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use edge_audit::InMemoryAuditLog;
    use edge_auth::{Claims, JwtKeyPair, RbacAuthorizer};
    use edge_decision::{DecisionOutcome, DecisionService};
    use edge_events::InMemoryEventBusBuilder;
    use edge_features::FeatureEngine;
    use edge_observability::{HealthRegistry, MetricsRegistry};
    use edge_rules::RuleEngine;
    use edge_transactions::{InMemoryDeduplicator, IngestionPipeline, RawTransaction};
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    fn test_app_state() -> (AppState, Arc<JwtKeyPair>) {
        let keypair = Arc::new(JwtKeyPair::generate());
        let authorizer = Arc::new(RbacAuthorizer::default());

        let audit = Arc::new(InMemoryAuditLog::new());
        let dedup = Arc::new(InMemoryDeduplicator::new());
        let bus = Arc::new(InMemoryEventBusBuilder::new().build());
        let ingestion = Arc::new(IngestionPipeline::new(dedup, bus, audit.clone()));

        let features = Arc::new(FeatureEngine::default());
        let rules = Arc::new(RuleEngine::new());
        let decision = Arc::new(DecisionService::new(features, rules, audit.clone()));

        let state = AppState {
            health_registry: Arc::new(HealthRegistry::new()),
            metrics_registry: Arc::new(MetricsRegistry::new()),
            keypair: keypair.clone(),
            authorizer,
            ingestion_pipeline: ingestion,
            decision_service: decision,
            audit_log: audit,
            recent_decisions: Arc::new(RecentDecisions::new()),
            active_rules_count: 4,
        };

        (state, keypair)
    }

    #[tokio::test]
    async fn test_liveness_endpoint() {
        let (state, _) = test_app_state();
        let app = build_router(state);
        let req = Request::builder()
            .uri("/health/liveness")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("x-frame-options").unwrap(), "DENY");
        assert_eq!(
            response.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );
    }

    #[tokio::test]
    async fn test_readiness_endpoint() {
        let (state, _) = test_app_state();
        let app = build_router(state);
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
        let (state, _) = test_app_state();
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
        let (state, _) = test_app_state();
        let app = build_router(state);
        let req = Request::builder()
            .uri("/openapi.json")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_submit_transaction_authorized() {
        let (state, keypair) = test_app_state();
        let token = keypair
            .sign(&Claims::new_access(
                Uuid::new_v4(),
                vec!["trader".to_string()],
                3600,
            ))
            .unwrap();

        let raw = RawTransaction {
            event_id: Some(Uuid::new_v4()),
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "payment".to_string(),
            channel: "web".to_string(),
            amount_minor: 5000,
            currency: "USD".to_string(),
            timestamp: None,
            metadata: None,
        };

        let app = build_router(state);
        let req = Request::builder()
            .uri("/api/v1/transactions")
            .method("POST")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&raw).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let outcome: DecisionOutcome = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(outcome.decision, edge_domain::Decision::Allow);
    }

    #[tokio::test]
    async fn test_submit_transaction_unauthorized_missing_token() {
        let (state, _) = test_app_state();
        let raw = RawTransaction {
            event_id: Some(Uuid::new_v4()),
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "payment".to_string(),
            channel: "web".to_string(),
            amount_minor: 5000,
            currency: "USD".to_string(),
            timestamp: None,
            metadata: None,
        };

        let app = build_router(state);
        let req = Request::builder()
            .uri("/api/v1/transactions")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&raw).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_submit_transaction_unauthorized_mismatched_key() {
        let (state, _) = test_app_state();
        let foreign_keypair = JwtKeyPair::generate();
        let token = foreign_keypair
            .sign(&Claims::new_access(
                Uuid::new_v4(),
                vec!["trader".to_string()],
                3600,
            ))
            .unwrap();

        let raw = RawTransaction {
            event_id: Some(Uuid::new_v4()),
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "payment".to_string(),
            channel: "web".to_string(),
            amount_minor: 5000,
            currency: "USD".to_string(),
            timestamp: None,
            metadata: None,
        };

        let app = build_router(state);
        let req = Request::builder()
            .uri("/api/v1/transactions")
            .method("POST")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&raw).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_submit_transaction_forbidden_insufficient_permissions() {
        let (state, keypair) = test_app_state();
        let token = keypair
            .sign(&Claims::new_access(
                Uuid::new_v4(),
                vec!["auditor".to_string()],
                3600,
            ))
            .unwrap();

        let raw = RawTransaction {
            event_id: Some(Uuid::new_v4()),
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "payment".to_string(),
            channel: "web".to_string(),
            amount_minor: 5000,
            currency: "USD".to_string(),
            timestamp: None,
            metadata: None,
        };

        let app = build_router(state);
        let req = Request::builder()
            .uri("/api/v1/transactions")
            .method("POST")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&raw).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_submit_transaction_conflict_duplicate() {
        let (state, keypair) = test_app_state();
        let token = keypair
            .sign(&Claims::new_access(
                Uuid::new_v4(),
                vec!["trader".to_string()],
                3600,
            ))
            .unwrap();

        let event_uuid = Uuid::new_v4();
        let raw = RawTransaction {
            event_id: Some(event_uuid),
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "payment".to_string(),
            channel: "web".to_string(),
            amount_minor: 5000,
            currency: "USD".to_string(),
            timestamp: None,
            metadata: None,
        };

        let app = build_router(state);

        let req1 = Request::builder()
            .uri("/api/v1/transactions")
            .method("POST")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&raw).unwrap()))
            .unwrap();

        let res1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(res1.status(), StatusCode::OK);

        let req2 = Request::builder()
            .uri("/api/v1/transactions")
            .method("POST")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&raw).unwrap()))
            .unwrap();

        let res2 = app.oneshot(req2).await.unwrap();
        assert_eq!(res2.status(), StatusCode::CONFLICT);
    }
}
