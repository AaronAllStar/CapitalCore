//! Axum router configuration and security middleware.

use crate::handlers::{
    get_audit_records_handler, get_decision_by_id_handler, get_decision_stats_handler,
    get_decisions_handler, get_demo_token_handler, get_model_info_handler, liveness_handler,
    metrics_handler, openapi_handler, readiness_handler, verify_audit_integrity_handler, AppState,
};
use crate::transaction::submit_transaction_handler;
use axum::body::Body;
use axum::http::{header, HeaderName, HeaderValue, Method, Request};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use edge_security::standard_security_headers;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Middleware applying standard production HTTP security headers to all responses.
pub async fn security_headers_middleware(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    for (name, val) in standard_security_headers() {
        if let (Ok(h_name), Ok(h_val)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(val),
        ) {
            headers.insert(h_name, h_val);
        }
    }
    response
}

/// Assembles the complete Axum HTTP application router.
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
        ]);

    Router::new()
        // Health and documentation
        .route("/health/liveness", get(liveness_handler))
        .route("/health/readiness", get(readiness_handler))
        .route("/metrics", get(metrics_handler))
        .route("/openapi.json", get(openapi_handler))
        // Transaction ingestion & assessment
        .route("/api/v1/transactions", post(submit_transaction_handler))
        // Decision queries & statistics
        .route("/api/v1/decisions", get(get_decisions_handler))
        .route("/api/v1/decisions/stats", get(get_decision_stats_handler))
        .route("/api/v1/decisions/:id", get(get_decision_by_id_handler))
        // Tamper-evident audit trail & verification
        .route("/api/v1/audit", get(get_audit_records_handler))
        .route("/api/v1/audit/verify", get(verify_audit_integrity_handler))
        // Machine Learning Model intelligence
        .route("/api/v1/model", get(get_model_info_handler))
        // Session and authorization tokens
        .route("/api/v1/auth/session", get(get_demo_token_handler))
        .route("/api/v1/auth/demo-token", get(get_demo_token_handler))
        .route("/api/v1/auth/token", post(get_demo_token_handler))
        // Middlewares
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
