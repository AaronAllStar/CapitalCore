//! CapitalCore Production Financial Decision & Fraud Prevention HTTP API Server.

use edge_api::{build_router, AppState, RecentDecisions, StoredDecision};
use edge_audit::InMemoryAuditLog;
use edge_auth::{JwtKeyPair, RbacAuthorizer};
use edge_decision::{DecisionService, MostRestrictive};
use edge_domain::{Decision, RuleId};
use edge_events::InMemoryEventBusBuilder;
use edge_features::FeatureEngine;
use edge_ml::FraudModel;
use edge_observability::{HealthRegistry, LogFormat, MetricsRegistry, ObservabilityConfig};
use edge_rules::{Condition, Operator, Rule, RuleAction, RuleEngine};
use edge_transactions::{InMemoryDeduplicator, IngestionPipeline, RawTransaction};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const MODEL_MANIFEST: &str = include_str!("../../../ml/models/model_manifest.json");

/// Decodes a 64-char hex string into a 32-byte Ed25519 seed.
fn parse_seed(hex: &str) -> Option<[u8; 32]> {
    let hex = hex.trim();
    if hex.len() != 64 || !hex.is_ascii() {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

/// Loads the JWT signing key from `JWT_SECRET_SEED` (64 hex chars).
/// In production the seed is mandatory; elsewhere an ephemeral key is generated.
fn load_keypair() -> Result<JwtKeyPair, Box<dyn std::error::Error>> {
    match std::env::var("JWT_SECRET_SEED") {
        Ok(raw) => parse_seed(&raw)
            .map(|seed| JwtKeyPair::from_secret_bytes(&seed))
            .ok_or_else(|| "JWT_SECRET_SEED must be exactly 64 hex characters".into()),
        Err(_) if std::env::var("APP_ENV").as_deref() == Ok("production") => {
            Err("JWT_SECRET_SEED is required when APP_ENV=production".into())
        }
        Err(_) => {
            tracing::warn!(
                "JWT_SECRET_SEED not set; using ephemeral key (tokens invalid after restart)"
            );
            Ok(JwtKeyPair::generate())
        }
    }
}

/// Resolves on SIGINT or (on Unix) SIGTERM, the signal Docker sends on stop.
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut term) = signal(SignalKind::terminate()) {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = term.recv() => {}
            }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}

/// Builds foundational deterministic banking fraud and AML compliance rules.
fn configure_banking_rules() -> (RuleEngine, usize) {
    let mut rules = RuleEngine::new();

    // Rule 1: High Single Transaction Review ($10,000+ = 1,000,000 minor units)
    rules.add_rule(Rule::new(
        RuleId::new(),
        "High Single Transaction Review",
        10,
        Condition::FeatureThreshold {
            feature: "amount_minor".to_string(),
            op: Operator::Gte,
            value: 1_000_000,
        },
        RuleAction::YieldDecision(Decision::Review),
        "RULE_HIGH_AMOUNT_REVIEW",
    ));

    // Rule 2: Massive Transaction Escalation ($50,000+ = 5,000,000 minor units)
    rules.add_rule(Rule::new(
        RuleId::new(),
        "Massive Transaction Escalation",
        5,
        Condition::FeatureThreshold {
            feature: "amount_minor".to_string(),
            op: Operator::Gte,
            value: 5_000_000,
        },
        RuleAction::YieldDecision(Decision::Escalate),
        "RULE_MASSIVE_AMOUNT_ESCALATE",
    ));

    // Rule 3: 5-Minute Velocity Surge Review (5+ txns in 5 min)
    rules.add_rule(Rule::new(
        RuleId::new(),
        "5-Minute High Velocity Burst",
        15,
        Condition::FeatureThreshold {
            feature: "user_txn_count_5m".to_string(),
            op: Operator::Gte,
            value: 5,
        },
        RuleAction::YieldDecision(Decision::Review),
        "RULE_VELOCITY_5M_BURST",
    ));

    // Rule 4: 1-Hour Aggregated Velocity Volume Block ($25,000+ within 1 hour)
    rules.add_rule(Rule::new(
        RuleId::new(),
        "1-Hour Velocity Aggregate Block",
        2,
        Condition::FeatureThreshold {
            feature: "user_txn_sum_1h".to_string(),
            op: Operator::Gte,
            value: 2_500_000,
        },
        RuleAction::YieldDecision(Decision::Block),
        "RULE_VELOCITY_1H_EXCEEDED",
    ));

    (rules, 4)
}

/// Seeds diverse realistic banking transactions into the platform pipeline.
async fn seed_initial_banking_data(
    ingestion: &IngestionPipeline,
    decision_svc: &DecisionService,
    store: &RecentDecisions,
) {
    let now = chrono::Utc::now();
    let user_alice = Uuid::parse_str("9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d").unwrap();
    let user_bob = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
    let user_charlie = Uuid::parse_str("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11").unwrap();
    let user_corp = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let samples = vec![
        // 1. Everyday low-risk retail payment ($42.50) -> ALLOW
        (
            user_alice,
            4250,
            "USD",
            "web",
            "payment",
            vec![
                ("merchant", "Amazon US"),
                ("category", "Retail"),
                ("ip_country", "US"),
            ],
            now - chrono::Duration::hours(5),
        ),
        // 2. Grocery Store contactless POS ($89.20) -> ALLOW
        (
            user_alice,
            8920,
            "USD",
            "pos",
            "payment",
            vec![
                ("merchant", "Whole Foods Market"),
                ("terminal_id", "POS-7741"),
                ("ip_country", "US"),
            ],
            now - chrono::Duration::hours(3),
        ),
        // 3. Subscription service monthly renewal ($14.99) -> ALLOW
        (
            user_bob,
            1499,
            "USD",
            "web",
            "payment",
            vec![
                ("merchant", "Spotify Premium"),
                ("category", "Digital Goods"),
                ("ip_country", "US"),
            ],
            now - chrono::Duration::hours(2),
        ),
        // 4. ATM Cash withdrawal ($200.00) -> ALLOW
        (
            user_bob,
            20000,
            "USD",
            "atm",
            "withdrawal",
            vec![
                ("atm_id", "ATM-NYC-084"),
                ("location", "Manhattan"),
                ("ip_country", "US"),
            ],
            now - chrono::Duration::hours(1),
        ),
        // 5. High-Value Mobile Wire Transfer ($12,500.00) -> REVIEW (Triggers Rule 1)
        (
            user_charlie,
            1250000,
            "USD",
            "mobile",
            "transfer",
            vec![
                ("recipient_bank", "JPMorgan Chase"),
                ("memo", "Vehicle Purchase"),
                ("ip_country", "US"),
            ],
            now - chrono::Duration::minutes(45),
        ),
        // 6. Massive Corporate Cross-Border Wire ($65,000.00) -> ESCALATE (Triggers Rule 2)
        (
            user_corp,
            6500000,
            "USD",
            "api",
            "transfer",
            vec![
                ("recipient_bank", "Barclays Bank UK"),
                ("iban", "GB82WEST12345698765432"),
                ("swift", "BARCGB22"),
            ],
            now - chrono::Duration::minutes(20),
        ),
        // 7. Velocity surge test - 5 rapid payments -> REVIEW (Triggers Rule 3)
        (
            user_charlie,
            15000,
            "USD",
            "mobile",
            "payment",
            vec![
                ("merchant", "Electronics Outlet"),
                ("device", "iOS App v4.2"),
            ],
            now - chrono::Duration::minutes(10),
        ),
        (
            user_charlie,
            18000,
            "USD",
            "mobile",
            "payment",
            vec![("merchant", "GameStop Digital"), ("device", "iOS App v4.2")],
            now - chrono::Duration::minutes(8),
        ),
        (
            user_charlie,
            22000,
            "USD",
            "mobile",
            "payment",
            vec![("merchant", "BestBuy Online"), ("device", "iOS App v4.2")],
            now - chrono::Duration::minutes(5),
        ),
        // 8. Suspected Carding Attack / High ML Fraud Risk ($3,850.00) -> BLOCK (Predicted fraud)
        (
            Uuid::new_v4(),
            385000,
            "USD",
            "web",
            "payment",
            vec![
                ("merchant", "Luxury Crypto Giftcards"),
                ("ip_country", "NG"),
                ("vpn_detected", "true"),
            ],
            now - chrono::Duration::minutes(2),
        ),
    ];

    for (user_id, amount_minor, currency, channel, event_type, meta, timestamp) in samples {
        let mut metadata = BTreeMap::new();
        for (k, v) in meta {
            metadata.insert(k.to_string(), v.to_string());
        }

        let raw = RawTransaction {
            event_id: None,
            transaction_id: Uuid::new_v4(),
            user_id,
            event_type: event_type.to_string(),
            channel: channel.to_string(),
            amount_minor,
            currency: currency.to_string(),
            timestamp: Some(timestamp),
            metadata: Some(metadata.clone()),
        };

        if let Ok(event) = ingestion.ingest(raw).await {
            if let Ok(outcome) = decision_svc.evaluate(&event).await {
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
                    metadata: metadata.clone(),
                    audit_id: outcome.audit_event.audit_id().as_uuid(),
                    created_at: event.created_at(),
                };
                store.record(stored);
            }
        }
    }

    info!("Pre-populated CapitalCore decision engine with realistic banking transactions");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("--version") => {
            println!("CapitalCore Decision API {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("--healthcheck") => {
            let port = std::env::var("PORT").unwrap_or_else(|_| "8001".to_string());
            return match std::net::TcpStream::connect_timeout(
                &format!("127.0.0.1:{port}").parse()?,
                std::time::Duration::from_secs(2),
            ) {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("healthcheck failed: {e}");
                    std::process::exit(1);
                }
            };
        }
        _ => {}
    }

    // 1. Initialize structured observability
    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let obs_config = ObservabilityConfig::new(log_filter, LogFormat::Json);
    let _ = obs_config.init_subscriber();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting CapitalCore Real-Time Financial Decision & Fraud Prevention API"
    );

    // 2. Initialize database connection pool & run migrations if configured
    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        let pool_config = edge_storage::PgPoolConfig::new(db_url);
        match pool_config.create_pool().await {
            Ok(pool) => {
                info!("Connected to PostgreSQL database pool");
                if let Err(e) = edge_storage::run_migrations(&pool).await {
                    tracing::error!(error = %e, "Failed executing database migrations");
                } else {
                    info!("Successfully ran database migrations");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Could not connect to PostgreSQL database pool; proceeding with in-memory adapters");
            }
        }
    }

    // 3. Initialize registries and core services
    let health_registry = Arc::new(HealthRegistry::new());
    let metrics_registry = Arc::new(MetricsRegistry::new());
    let keypair = Arc::new(load_keypair()?);
    let authorizer = Arc::new(RbacAuthorizer::default());

    let audit = Arc::new(InMemoryAuditLog::new());
    let dedup = Arc::new(InMemoryDeduplicator::new());
    let bus = Arc::new(InMemoryEventBusBuilder::new().build());
    let ingestion_pipeline = Arc::new(IngestionPipeline::new(dedup, bus, audit.clone()));

    let features = Arc::new(FeatureEngine::default());
    let (rules, active_rules_count) = configure_banking_rules();
    let rules = Arc::new(rules);

    let decision_service = match FraudModel::load_from_json(MODEL_MANIFEST) {
        Ok(model) => {
            info!("Successfully embedded machine learning fraud detection model");
            Arc::new(
                DecisionService::new(features, rules, audit.clone())
                    .with_arbitration(Arc::new(MostRestrictive))
                    .with_ml_model(Arc::new(model)),
            )
        }
        Err(err) => {
            tracing::warn!(error = %err, "Failed loading ML model manifest, falling back to rule-only engine");
            Arc::new(
                DecisionService::new(features, rules, audit.clone())
                    .with_arbitration(Arc::new(MostRestrictive)),
            )
        }
    };

    let recent_decisions = Arc::new(RecentDecisions::new());

    // 4. Seed initial banking sample transactions
    seed_initial_banking_data(&ingestion_pipeline, &decision_service, &recent_decisions).await;

    let state = AppState {
        health_registry,
        metrics_registry,
        keypair,
        authorizer,
        ingestion_pipeline,
        decision_service,
        audit_log: audit,
        recent_decisions,
        active_rules_count,
    };

    // 5. Assemble HTTP router
    let app = build_router(state);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8001".to_string())
        .parse::<u16>()
        .unwrap_or(8001);
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    info!(addr = %addr, "CapitalCore HTTP API listening for connections");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_signal().await;
            info!("Received shutdown signal; draining connections");
        })
        .await?;

    info!("CapitalCore HTTP API shutdown completed cleanly");
    Ok(())
}
