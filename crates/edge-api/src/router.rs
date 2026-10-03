//! Axum router configuration and security middleware.

use crate::handlers::{
    liveness_handler, metrics_handler, openapi_handler, readiness_handler, AppState,
};
use crate::transaction::submit_transaction_handler;
use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, Request};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use edge_security::standard_security_headers;
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
    Router::new()
        .route("/health/liveness", get(liveness_handler))
        .route("/health/readiness", get(readiness_handler))
        .route("/metrics", get(metrics_handler))
        .route("/openapi.json", get(openapi_handler))
        .route("/api/v1/transactions", post(submit_transaction_handler))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
