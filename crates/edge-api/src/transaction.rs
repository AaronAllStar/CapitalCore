//! Transaction submission HTTP route handler.

use crate::auth::authenticate_header;
use crate::handlers::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use edge_auth::permissions;
use edge_security::ErrorResponse;
use edge_transactions::{RawTransaction, TransactionError};

/// Submits an incoming financial transaction for ingestion and decision assessment.
pub async fn submit_transaction_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(raw): Json<RawTransaction>,
) -> impl IntoResponse {
    // 1. Authenticate JWT Bearer token
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());
    let user = match authenticate_header(auth_header, &state.keypair) {
        Ok(u) => u,
        Err(err) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::to_value(err).unwrap()),
            )
        }
    };

    // 2. Authorize RBAC permission
    if let Err(err) = user.authorize(&state.authorizer, permissions::EVENTS_WRITE) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::to_value(err).unwrap()),
        );
    }

    // 3. Ingest and validate transaction via pipeline
    let event = match state.ingestion_pipeline.ingest(raw).await {
        Ok(ev) => ev,
        Err(TransactionError::Duplicate(msg)) => {
            let resp = ErrorResponse::new("ERR_DUPLICATE_EVENT", msg);
            return (
                StatusCode::CONFLICT,
                Json(serde_json::to_value(resp).unwrap()),
            );
        }
        Err(err) => {
            let resp = ErrorResponse::bad_request(err.to_string());
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::to_value(resp).unwrap()),
            );
        }
    };

    // 4. Evaluate decision
    match state.decision_service.evaluate(&event).await {
        Ok(outcome) => (StatusCode::OK, Json(serde_json::to_value(outcome).unwrap())),
        Err(err) => {
            let resp = ErrorResponse::internal_error(err.to_string());
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::to_value(resp).unwrap()),
            )
        }
    }
}
