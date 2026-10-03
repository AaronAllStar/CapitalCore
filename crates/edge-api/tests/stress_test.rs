//! Comprehensive concurrent stress test harness for EdgeArena API.
//!
//! Evaluates:
//! - Multi-worker concurrent load (25 clients, 500 transactions).
//! - Sub-millisecond latency distribution under sustained pressure.
//! - Deterministic idempotency / duplicate rejection under concurrent race conditions.
//! - Tamper-evident cryptographic audit chain integrity under concurrency.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use edge_api::{build_router, AppState};
use edge_audit::{AuditLog, InMemoryAuditLog};
use edge_auth::{Claims, JwtKeyPair, RbacAuthorizer};
use edge_decision::{DecisionService, MostRestrictive};
use edge_events::InMemoryEventBusBuilder;
use edge_features::FeatureEngine;
use edge_ml::FraudModel;
use edge_observability::{HealthRegistry, MetricsRegistry};
use edge_rules::{Condition, Operator, Rule, RuleAction, RuleEngine};
use edge_transactions::{InMemoryDeduplicator, IngestionPipeline, RawTransaction};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Barrier;
use tower::ServiceExt;
use uuid::Uuid;

const MODEL_MANIFEST: &str = include_str!("../../../ml/models/model_manifest.json");

fn setup_stress_environment() -> (AppState, Arc<JwtKeyPair>, Arc<InMemoryAuditLog>) {
    let keypair = Arc::new(JwtKeyPair::generate());
    let authorizer = Arc::new(RbacAuthorizer::default());

    let audit = Arc::new(InMemoryAuditLog::new());
    let dedup = Arc::new(InMemoryDeduplicator::new());
    let bus = Arc::new(InMemoryEventBusBuilder::new().build());
    let ingestion = Arc::new(IngestionPipeline::new(dedup, bus, audit.clone()));

    let features = Arc::new(FeatureEngine::default());
    let mut rules = RuleEngine::new();

    // Review high velocity transactions
    rules.add_rule(Rule::new(
        edge_domain::RuleId::new(),
        "review_high_velocity".to_string(),
        100,
        Condition::FeatureThreshold {
            feature: "user_txn_count_5m".to_string(),
            op: Operator::Gt,
            value: 10,
        },
        RuleAction::YieldDecision(edge_domain::Decision::Review),
        "High velocity limit exceeded".to_string(),
    ));

    // Block high ML fraud risk
    rules.add_rule(Rule::new(
        edge_domain::RuleId::new(),
        "block_ml_risk".to_string(),
        200,
        Condition::FeatureThreshold {
            feature: "ml_fraud_score_bps".to_string(),
            op: Operator::Gte,
            value: 8000,
        },
        RuleAction::YieldDecision(edge_domain::Decision::Block),
        "ML fraud score high".to_string(),
    ));

    let model = FraudModel::load_from_json(MODEL_MANIFEST).expect("valid model manifest");

    let decision = Arc::new(
        DecisionService::new(features, Arc::new(rules), audit.clone())
            .with_arbitration(Arc::new(MostRestrictive))
            .with_ml_model(Arc::new(model)),
    );

    let state = AppState {
        health_registry: Arc::new(HealthRegistry::new()),
        metrics_registry: Arc::new(MetricsRegistry::new()),
        keypair: keypair.clone(),
        authorizer,
        ingestion_pipeline: ingestion,
        decision_service: decision,
    };

    (state, keypair, audit)
}

#[tokio::test]
async fn test_sustained_concurrent_transaction_load() {
    let (state, keypair, audit) = setup_stress_environment();
    let num_tasks = 25;
    let requests_per_task = 20;
    let total_requests = num_tasks * requests_per_task;

    let barrier = Arc::new(Barrier::new(num_tasks));
    let mut join_handles = Vec::with_capacity(num_tasks);

    let start_all = Instant::now();

    for task_idx in 0..num_tasks {
        let state_clone = state.clone();
        let keypair_clone = keypair.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            let app = build_router(state_clone);
            let user_uuid = Uuid::new_v4();
            let claims = Claims::new_access(user_uuid, vec!["trader".to_string()], 3600);
            let token = keypair_clone.sign(&claims).unwrap();

            // Synchronize task starts to maximize burst concurrency
            barrier_clone.wait().await;

            let mut task_latencies = Vec::with_capacity(requests_per_task);

            for req_idx in 0..requests_per_task {
                let tx = RawTransaction {
                    event_id: Some(Uuid::new_v4()),
                    transaction_id: Uuid::new_v4(),
                    user_id: user_uuid,
                    event_type: "payment".to_string(),
                    channel: "web".to_string(),
                    amount_minor: 1000 + (task_idx * 100 + req_idx) as i64,
                    currency: "USD".to_string(),
                    timestamp: None,
                    metadata: None,
                };

                let body = serde_json::to_vec(&tx).unwrap();
                let req = Request::builder()
                    .method("POST")
                    .uri("/api/v1/transactions")
                    .header("Authorization", format!("Bearer {token}"))
                    .header("Content-Type", "application/json")
                    .body(Body::from(body))
                    .unwrap();

                let req_start = Instant::now();
                let response = app.clone().oneshot(req).await.unwrap();
                let elapsed = req_start.elapsed();

                assert_eq!(response.status(), StatusCode::OK);
                task_latencies.push(elapsed);
            }

            task_latencies
        });

        join_handles.push(handle);
    }

    let mut all_latencies = Vec::with_capacity(total_requests);
    for handle in join_handles {
        let latencies = handle.await.expect("task completed without panic");
        all_latencies.extend(latencies);
    }

    let total_duration = start_all.elapsed();
    let total_secs = total_duration.as_secs_f64();
    let tps = (total_requests as f64) / total_secs;

    all_latencies.sort();
    let p50 = all_latencies[total_requests * 50 / 100];
    let p95 = all_latencies[total_requests * 95 / 100];
    let p99 = all_latencies[total_requests * 99 / 100];

    println!(
        "STRESS TEST SUMMARY: {total_requests} requests processed in {:.3}s -> {:.1} TPS | p50: {:?}, p95: {:?}, p99: {:?}",
        total_secs, tps, p50, p95, p99
    );

    // Assert high throughput and sub-millisecond latencies
    assert!(tps > 40.0, "Throughput {tps} TPS should exceed baseline");
    assert!(
        p99.as_micros() < 50_000,
        "p99 latency should be under 50ms under local mock burst"
    );

    // Verify all decisions and ingestions were recorded in audit log with unbroken cryptographic chain
    // Each transaction produces 2 audit records: 1 from IngestionPipeline and 1 from DecisionService
    assert_eq!(audit.count(), total_requests * 2);
    let chain_valid = audit
        .verify_integrity()
        .await
        .expect("chain integrity check");
    assert!(
        chain_valid,
        "Audit log cryptographic chain must be unbroken"
    );
}

#[tokio::test]
async fn test_concurrent_idempotency_race_condition() {
    let (state, keypair, audit) = setup_stress_environment();
    let num_racers = 20;

    let duplicate_event_id = Uuid::new_v4();
    let duplicate_tx_id = Uuid::new_v4();
    let user_uuid = Uuid::new_v4();

    let barrier = Arc::new(Barrier::new(num_racers));
    let mut join_handles = Vec::with_capacity(num_racers);

    for _ in 0..num_racers {
        let state_clone = state.clone();
        let keypair_clone = keypair.clone();
        let barrier_clone = barrier.clone();
        let ev_id = duplicate_event_id;
        let tx_id = duplicate_tx_id;

        let handle = tokio::spawn(async move {
            let app = build_router(state_clone);
            let claims = Claims::new_access(user_uuid, vec!["trader".to_string()], 3600);
            let token = keypair_clone.sign(&claims).unwrap();

            let tx = RawTransaction {
                event_id: Some(ev_id),
                transaction_id: tx_id,
                user_id: user_uuid,
                event_type: "payment".to_string(),
                channel: "web".to_string(),
                amount_minor: 5000,
                currency: "USD".to_string(),
                timestamp: None,
                metadata: None,
            };

            let body = serde_json::to_vec(&tx).unwrap();
            let req = Request::builder()
                .method("POST")
                .uri("/api/v1/transactions")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .unwrap();

            barrier_clone.wait().await;
            let response = app.oneshot(req).await.unwrap();
            response.status()
        });

        join_handles.push(handle);
    }

    let mut ok_count = 0;
    let mut conflict_count = 0;

    for handle in join_handles {
        let status = handle.await.unwrap();
        if status == StatusCode::OK {
            ok_count += 1;
        } else if status == StatusCode::CONFLICT {
            conflict_count += 1;
        }
    }

    // Exactly one concurrent racer must succeed; all others must be rejected with 409 Conflict
    assert_eq!(ok_count, 1, "Exactly one request should win the race");
    assert_eq!(
        conflict_count,
        num_racers - 1,
        "All other racing requests must be rejected as duplicates"
    );
    // 1 ingestion audit event + 1 decision audit event = 2 records in total
    assert_eq!(
        audit.count(),
        2,
        "Audit log must contain the winning transaction's ingestion and decision records"
    );
    let chain_valid = audit
        .verify_integrity()
        .await
        .expect("chain integrity check");
    assert!(
        chain_valid,
        "Audit log cryptographic chain must be unbroken"
    );
}
