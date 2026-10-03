use chrono::{Duration, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use edge_audit::InMemoryAuditLog;
use edge_core::{Currency, Money};
use edge_decision::{DecisionService, MostRestrictive};
use edge_domain::{
    Channel, Decision, EventId, EventType, FinancialEvent, RuleId, TransactionId, UserId,
};
use edge_features::FeatureEngine;
use edge_ml::FraudModel;
use edge_rules::{Condition, Operator, Rule, RuleAction, RuleEngine};
use edge_transactions::{Deduplicator, InMemoryDeduplicator};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

const MODEL_MANIFEST: &str = include_str!("../../../ml/models/model_manifest.json");

fn make_benchmark_event(
    user_id: UserId,
    amount: i64,
    time: chrono::DateTime<Utc>,
) -> FinancialEvent {
    let mut metadata = BTreeMap::new();
    metadata.insert("client_ip".to_string(), "10.0.0.1".to_string());
    FinancialEvent::new(
        EventId::new(),
        TransactionId::new(),
        user_id,
        EventType::Payment,
        Channel::Web,
        Money::new(amount, Currency::USD),
        time,
        metadata,
    )
}

fn bench_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("feature_engine");
    let user_id = UserId::new();
    let now = Utc::now();

    group.bench_function("window_record_and_extract", |b| {
        let engine = FeatureEngine::default();
        let mut t = now;
        b.iter(|| {
            t += Duration::seconds(1);
            let event = make_benchmark_event(user_id, 2500, t);
            let features = engine.compute_and_update(black_box(&event));
            black_box(features)
        })
    });

    group.finish();
}

fn bench_ml(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_inference");
    let model = FraudModel::load_from_json(MODEL_MANIFEST).expect("valid model manifest");

    let mut features = BTreeMap::new();
    features.insert("amount_minor".to_string(), 5000);
    features.insert("user_txn_count_5m".to_string(), 3);
    features.insert("user_txn_sum_5m".to_string(), 15000);
    features.insert("user_txn_count_1h".to_string(), 12);
    features.insert("user_txn_sum_1h".to_string(), 60000);
    features.insert("user_txn_max_1h".to_string(), 25000);
    features.insert("user_txn_count_24h".to_string(), 45);
    features.insert("user_txn_sum_24h".to_string(), 180000);
    features.insert("user_distinct_merchants_24h".to_string(), 5);

    group.bench_function("fraud_model_predict_score", |b| {
        b.iter(|| {
            let score = model.predict_score(black_box(&features)).unwrap();
            black_box(score)
        })
    });

    group.bench_function("fraud_model_predict_risk_bps", |b| {
        b.iter(|| {
            let bps = model.predict_risk_bps(black_box(&features)).unwrap();
            black_box(bps)
        })
    });

    group.finish();
}

fn bench_deduplication(c: &mut Criterion) {
    let mut group = c.benchmark_group("ingestion_deduplication");
    let dedup = InMemoryDeduplicator::new();
    let ev_id = EventId::new();

    group.bench_function("check_and_record_new", |b| {
        b.iter(|| {
            let id = EventId::new();
            black_box(dedup.check_and_record(black_box(&id)))
        })
    });

    let _ = dedup.check_and_record(&ev_id);
    group.bench_function("check_and_record_duplicate", |b| {
        b.iter(|| black_box(dedup.check_and_record(black_box(&ev_id))))
    });

    group.finish();
}

fn bench_end_to_end_decision(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_decision_pipeline");
    let rt = Runtime::new().unwrap();

    let features = Arc::new(FeatureEngine::default());
    let mut rules = RuleEngine::new();

    // High velocity review rule
    rules.add_rule(Rule::new(
        RuleId::new(),
        "review_high_velocity".to_string(),
        100,
        Condition::FeatureThreshold {
            feature: "user_txn_count_5m".to_string(),
            op: Operator::Gt,
            value: 5,
        },
        RuleAction::YieldDecision(Decision::Review),
        "High 5m transaction velocity".to_string(),
    ));

    // High risk ML score block rule
    rules.add_rule(Rule::new(
        RuleId::new(),
        "block_ml_high_risk".to_string(),
        200,
        Condition::FeatureThreshold {
            feature: "ml_fraud_score_bps".to_string(),
            op: Operator::Gte,
            value: 7500,
        },
        RuleAction::YieldDecision(Decision::Block),
        "ML fraud score exceeds 75% threshold".to_string(),
    ));

    let audit = Arc::new(InMemoryAuditLog::new());
    let model = FraudModel::load_from_json(MODEL_MANIFEST).expect("valid model manifest");

    let service = DecisionService::new(features, Arc::new(rules), audit)
        .with_arbitration(Arc::new(MostRestrictive))
        .with_ml_model(Arc::new(model));

    let user_id = UserId::new();
    let mut now = Utc::now();

    group.bench_function("full_decision_evaluation", |b| {
        b.iter(|| {
            now += Duration::milliseconds(100);
            let event = make_benchmark_event(user_id, 4500, now);
            let outcome = rt.block_on(async { service.evaluate(black_box(&event)).await.unwrap() });
            black_box(outcome)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_features,
    bench_ml,
    bench_deduplication,
    bench_end_to_end_decision
);
criterion_main!(benches);
