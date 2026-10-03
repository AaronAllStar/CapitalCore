//! Core Axum router assembly.

use crate::handlers::{
    liveness_handler, metrics_handler, openapi_handler, readiness_handler, AppState,
};
use crate::transaction::submit_transaction_handler;
use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

/// Assembles the complete Axum HTTP application router.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health/liveness", get(liveness_handler))
        .route("/health/readiness", get(readiness_handler))
        .route("/metrics", get(metrics_handler))
        .route("/openapi.json", get(openapi_handler))
        .route("/api/v1/transactions", post(submit_transaction_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
