use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use edge_audit::AuditRecord;
use edge_core::{Currency, Money, RoundingMode};
use edge_domain::{
    AuditEvent, AuditId, Channel, Decision, EventId, EventType, FinancialEvent, TransactionId,
    UserId,
};
use std::collections::BTreeMap;

fn bench_money(c: &mut Criterion) {
    let mut group = c.benchmark_group("money_arithmetic");

    let m1 = Money::new(100_000, Currency::USD);
    let m2 = Money::new(50_000, Currency::USD);

    group.bench_function("checked_add", |b| {
        b.iter(|| black_box(m1).checked_add(black_box(m2)))
    });

    group.bench_function("checked_div_half_even", |b| {
        b.iter(|| black_box(m1).checked_div(black_box(3), RoundingMode::HalfEven))
    });

    group.finish();
}

fn bench_event_serde(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_serde");

    let mut metadata = BTreeMap::new();
    metadata.insert("ip".to_string(), "192.168.1.1".to_string());
    metadata.insert("country".to_string(), "US".to_string());

    let event = FinancialEvent::new(
        EventId::new(),
        TransactionId::new(),
        UserId::new(),
        EventType::Payment,
        Channel::Web,
        Money::new(1500, Currency::USD),
        Utc::now(),
        metadata,
    );

    let serialized = serde_json::to_string(&event).unwrap();

    group.bench_function("serialize_financial_event", |b| {
        b.iter(|| serde_json::to_string(black_box(&event)))
    });

    group.bench_function("deserialize_financial_event", |b| {
        b.iter(|| serde_json::from_str::<FinancialEvent>(black_box(&serialized)))
    });

    group.finish();
}

fn bench_audit(c: &mut Criterion) {
    let mut group = c.benchmark_group("audit_operations");

    let mut feature_versions = BTreeMap::new();
    feature_versions.insert("txn_freq_24h".to_string(), "1.0.0".to_string());

    let mut rule_versions = BTreeMap::new();
    rule_versions.insert("block_high_velocity".to_string(), "1.0.0".to_string());

    let event = AuditEvent::new(
        AuditId::new(),
        EventId::new(),
        Decision::Block,
        vec!["Velocity threshold exceeded".to_string()],
        feature_versions,
        rule_versions,
        Some("model-v1".to_string()),
        Utc::now(),
    );

    group.bench_function("audit_record_hash_calculation", |b| {
        b.iter(|| {
            AuditRecord::calculate_hash(
                black_box(1),
                black_box("0000000000000000"),
                black_box(&event),
            )
        })
    });

    group.finish();
}

criterion_group!(benches, bench_money, bench_event_serde, bench_audit);
criterion_main!(benches);
