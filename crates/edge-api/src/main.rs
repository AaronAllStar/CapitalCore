//! EdgeArena Production HTTP API Server Entry Point.

use edge_api::{build_router, AppState};
use edge_audit::InMemoryAuditLog;
use edge_auth::{JwtKeyPair, RbacAuthorizer};
use edge_decision::{DecisionService, MostRestrictive};
use edge_events::InMemoryEventBusBuilder;
use edge_features::FeatureEngine;
use edge_ml::FraudModel;
use edge_observability::{HealthRegistry, LogFormat, MetricsRegistry, ObservabilityConfig};
use edge_rules::RuleEngine;
use edge_transactions::{InMemoryDeduplicator, IngestionPipeline};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

const MODEL_MANIFEST: &str = include_str!("../../../ml/models/model_manifest.json");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize structured observability
    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let obs_config = ObservabilityConfig::new(log_filter, LogFormat::Json);
    let _ = obs_config.init_subscriber();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting EdgeArena core decision API"
    );

    // 2. Initialize registries and core services
    let health_registry = Arc::new(HealthRegistry::new());
    let metrics_registry = Arc::new(MetricsRegistry::new());
    let keypair = Arc::new(JwtKeyPair::generate());
    let authorizer = Arc::new(RbacAuthorizer::default());

    let audit = Arc::new(InMemoryAuditLog::new());
    let dedup = Arc::new(InMemoryDeduplicator::new());
    let bus = Arc::new(InMemoryEventBusBuilder::new().build());
    let ingestion_pipeline = Arc::new(IngestionPipeline::new(dedup, bus, audit.clone()));

    let features = Arc::new(FeatureEngine::default());
    let rules = Arc::new(RuleEngine::new());

    let decision_service = match FraudModel::load_from_json(MODEL_MANIFEST) {
        Ok(model) => {
            info!("Successfully embedded machine learning fraud detection model");
            Arc::new(
                DecisionService::new(features, rules, audit)
                    .with_arbitration(Arc::new(MostRestrictive))
                    .with_ml_model(Arc::new(model)),
            )
        }
        Err(err) => {
            tracing::warn!(error = %err, "Failed loading ML model manifest, falling back to rule-only engine");
            Arc::new(
                DecisionService::new(features, rules, audit)
                    .with_arbitration(Arc::new(MostRestrictive)),
            )
        }
    };

    let state = AppState {
        health_registry,
        metrics_registry,
        keypair,
        authorizer,
        ingestion_pipeline,
        decision_service,
    };

    // 3. Assemble HTTP router
    let app = build_router(state);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    info!(addr = %addr, "EdgeArena HTTP API listening for connections");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
            info!("Received shutdown signal; draining connections");
        })
        .await?;

    info!("EdgeArena HTTP API shutdown completed cleanly");
    Ok(())
}
