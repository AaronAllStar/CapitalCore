use crate::decision_store::RecentDecisions;
use crate::openapi::generate_openapi_spec;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use edge_audit::AuditLog;
use edge_auth::{Claims, JwtKeyPair, RbacAuthorizer};
use edge_decision::DecisionService;
use edge_observability::{HealthRegistry, HealthStatus, MetricsRegistry};
use edge_transactions::IngestionPipeline;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// Shared application state for foundational endpoints.
#[derive(Clone)]
pub struct AppState {
    /// Registry of subsystem health checks.
    pub health_registry: Arc<HealthRegistry>,
    /// Thread-safe metrics registry.
    pub metrics_registry: Arc<MetricsRegistry>,
    /// Cryptographic JWT key pair for verifying authentication tokens.
    pub keypair: Arc<JwtKeyPair>,
    /// Role-based access control authorizer.
    pub authorizer: Arc<RbacAuthorizer>,
    /// Transaction ingestion pipeline.
    pub ingestion_pipeline: Arc<IngestionPipeline>,
    /// Central decision engine service.
    pub decision_service: Arc<DecisionService>,
    /// Audit log engine.
    pub audit_log: Arc<dyn AuditLog>,
    /// Live queryable decisions store.
    pub recent_decisions: Arc<RecentDecisions>,
    /// Number of active deterministic rules.
    pub active_rules_count: usize,
}

/// Liveness probe handler returning HTTP 200 OK.
pub async fn liveness_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({ "status": "UP", "platform": "CapitalCore" })),
    )
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

/// Query parameters for filtering decisions.
#[derive(Debug, Deserialize)]
pub struct DecisionFilterQuery {
    /// Optional decision filter (ALLOW, REVIEW, ESCALATE, BLOCK).
    pub status: Option<String>,
    /// Maximum number of records to return (defaults to 50, max 200).
    pub limit: Option<usize>,
}

/// Handler listing recent evaluated decisions.
pub async fn get_decisions_handler(
    State(state): State<AppState>,
    Query(query): Query<DecisionFilterQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).min(200);
    let decisions = state.recent_decisions.list(query.status.as_deref(), limit);
    (StatusCode::OK, Json(decisions))
}

/// Handler retrieving real-time statistics and KPI counters.
pub async fn get_decision_stats_handler(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.recent_decisions.stats(state.active_rules_count);
    (StatusCode::OK, Json(stats))
}

/// Handler retrieving a specific decision by ID.
pub async fn get_decision_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.recent_decisions.find_by_id(id) {
        Some(decision) => (StatusCode::OK, Json(json!(decision))),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Decision record not found", "id": id })),
        ),
    }
}

/// Handler executing cryptographic integrity verification of the append-only audit hash chain.
pub async fn verify_audit_integrity_handler(State(state): State<AppState>) -> impl IntoResponse {
    match state.audit_log.verify_integrity().await {
        Ok(is_valid) => (
            StatusCode::OK,
            Json(json!({
                "valid": is_valid,
                "chain_status": if is_valid { "VERIFIED_TAMPER_EVIDENT" } else { "INTEGRITY_COMPROMISED" },
                "verified_at": Utc::now()
            })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "valid": false,
                "chain_status": "INTEGRITY_VIOLATION",
                "error": err.to_string(),
                "verified_at": Utc::now()
            })),
        ),
    }
}

/// Handler returning recent audit log events within the last 30 days.
pub async fn get_audit_records_handler(State(state): State<AppState>) -> impl IntoResponse {
    let start = Utc::now() - Duration::days(30);
    let end = Utc::now() + Duration::hours(1);

    match state.audit_log.query_range(start, end).await {
        Ok(events) => (StatusCode::OK, Json(json!(events))),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        ),
    }
}

/// Handler returning CapitalCore Machine Learning Fraud Detection model specs.
pub async fn get_model_info_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "model_name": "CapitalCore Anti-Fraud Gradient Decision Tree",
            "version": "v1.2.0-onnx",
            "runtime": "Embedded Rust Native (edge-ml)",
            "accuracy": "99.60%",
            "precision": "96.55%",
            "recall": "100.00%",
            "f1_score": "98.25%",
            "latency_p99_us": 12.4,
            "arbitration_strategy": "MostRestrictive (Block > Escalate > Review > Allow)",
            "features_catalog": [
                { "name": "amount_minor", "type": "int64", "window": "instant", "description": "Transaction amount in minor units" },
                { "name": "user_txn_count_5m", "type": "int64", "window": "5 minutes", "description": "Trailing transaction frequency" },
                { "name": "user_txn_sum_5m", "type": "int64", "window": "5 minutes", "description": "Trailing aggregate volume" },
                { "name": "user_txn_count_1h", "type": "int64", "window": "1 hour", "description": "Trailing transaction count" },
                { "name": "user_txn_sum_1h", "type": "int64", "window": "1 hour", "description": "Trailing aggregate volume" },
                { "name": "user_txn_max_1h", "type": "int64", "window": "1 hour", "description": "Peak single transaction amount" },
                { "name": "user_txn_count_24h", "type": "int64", "window": "24 hours", "description": "Daily transaction count" },
                { "name": "user_txn_sum_24h", "type": "int64", "window": "24 hours", "description": "Daily cumulative volume" },
                { "name": "user_distinct_merchants_24h", "type": "int64", "window": "24 hours", "description": "Distinct merchants visited" }
            ],
            "thresholds": {
                "fraud_cutoff_bps": 5000,
                "review_cutoff_bps": 2500
            }
        })),
    )
}

/// Handler issuing an Ed25519 JWT session token for the CapitalCore Bank Analyst console.
pub async fn get_demo_token_handler(State(state): State<AppState>) -> impl IntoResponse {
    let analyst_id = Uuid::new_v4();
    let claims = Claims::new_access(
        analyst_id,
        vec!["analyst".to_string(), "admin".to_string()],
        86400, // 24 hours
    );

    match state.keypair.sign(&claims) {
        Ok(token) => (
            StatusCode::OK,
            Json(json!({
                "access_token": token,
                "token_type": "Bearer",
                "expires_in": 86400,
                "user": {
                    "id": analyst_id,
                    "name": "Lead Risk Analyst",
                    "email": "analyst@capitalcore.bank",
                    "role": "Risk Analyst",
                    "department": "Fraud Prevention & Compliance",
                    "permissions": [
                        "events:read",
                        "events:write",
                        "decisions:read",
                        "rules:read",
                        "audit:read"
                    ]
                }
            })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to sign token: {err}") })),
        ),
    }
}
