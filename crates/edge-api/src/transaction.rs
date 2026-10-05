//! Transaction submission HTTP route handler.

use crate::auth::authenticate_header;
use crate::decision_store::StoredDecision;
use crate::handlers::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use edge_auth::permissions;
use edge_security::ErrorResponse;
use edge_transactions::{RawTransaction, TransactionError};
use uuid::Uuid;

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
        Ok(outcome) => {
            // Record in the live decision store for analyst console querying
            let stored = StoredDecision {
                id: Uuid::new_v4(),
                event_id: event.event_id().as_uuid(),
                transaction_id: event.transaction_id().as_uuid(),
                user_id: event.user_id().as_uuid(),
                amount_minor: event.money().minor_units(),
                currency: event.money().currency().to_string(),
                channel: event.channel().as_str().to_string(),
                event_type: event.event_type().as_str().to_string(),
                decision: outcome.decision.as_str().to_string(),
                reasons: outcome.reasons.clone(),
                risk_score_bps: outcome.features_snapshot.get("ml_fraud_score_bps").copied(),
                is_fraud_predicted: outcome
                    .features_snapshot
                    .get("ml_is_fraud")
                    .map(|&v| v == 1),
                model_version: outcome.audit_event.model_version().map(|s| s.to_string()),
                features_snapshot: outcome.features_snapshot.clone(),
                metadata: event.metadata().clone(),
                audit_id: outcome.audit_event.audit_id().as_uuid(),
                created_at: event.created_at(),
            };
            state.recent_decisions.record(stored);

            (StatusCode::OK, Json(serde_json::to_value(outcome).unwrap()))
        }
        Err(err) => {
            let resp = ErrorResponse::internal_error(err.to_string());
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::to_value(resp).unwrap()),
            )
        }
    }
}
