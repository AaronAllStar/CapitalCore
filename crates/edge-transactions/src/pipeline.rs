//! Orchestrated transaction ingestion pipeline combining validation, deduplication, audit logging, and bus publication.

use crate::dedup::Deduplicator;
use crate::error::TransactionError;
use crate::normalization::{normalize_transaction, RawTransaction};
use chrono::Utc;
use edge_audit::AuditLog;
use edge_domain::{AuditEvent, AuditId, Decision, FinancialEvent};
use edge_events::EventBus;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Asynchronous pipeline for ingesting, validating, deduplicating, and dispatching financial transactions.
pub struct IngestionPipeline {
    deduplicator: Arc<dyn Deduplicator>,
    event_bus: Arc<dyn EventBus>,
    audit_log: Arc<dyn AuditLog>,
}

impl IngestionPipeline {
    /// Creates a new `IngestionPipeline` with the given service dependencies.
    pub fn new(
        deduplicator: Arc<dyn Deduplicator>,
        event_bus: Arc<dyn EventBus>,
        audit_log: Arc<dyn AuditLog>,
    ) -> Self {
        Self {
            deduplicator,
            event_bus,
            audit_log,
        }
    }

    /// Ingests a raw transaction, executing normalization, deduplication, audit logging, and bus dispatch.
    pub async fn ingest(&self, raw: RawTransaction) -> Result<FinancialEvent, TransactionError> {
        // Step 1: Normalize and validate raw transaction payload
        let event = normalize_transaction(raw)?;

        // Step 2: Enforce deduplication / idempotency
        self.deduplicator.check_and_record(&event.event_id())?;

        // Step 3: Record ingestion audit record
        let audit = AuditEvent::new(
            AuditId::new(),
            event.event_id(),
            Decision::Allow,
            vec!["INGESTION_ACCEPTED".to_string()],
            BTreeMap::new(),
            BTreeMap::new(),
            None,
            Utc::now(),
        );
        self.audit_log
            .append(audit)
            .await
            .map_err(|e| TransactionError::Audit(e.to_string()))?;

        // Step 4: Dispatch to the event bus for downstream processing
        self.event_bus
            .publish(&event)
            .await
            .map_err(|e| TransactionError::Bus(e.to_string()))?;

        Ok(event)
    }
}
