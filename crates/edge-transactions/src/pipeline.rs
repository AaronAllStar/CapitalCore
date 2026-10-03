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
use uuid::Uuid;

/// Policy trait for deciding whether a normalized financial event should be processed or bypassed.
pub trait TransactionFilter: Send + Sync {
    /// Returns `true` if the event should continue through ingestion and bus dispatch.
    fn should_process(&self, event: &FinancialEvent) -> bool;
}

/// Aggregated outcome for batch transaction ingestion requests.
#[derive(Debug, Default)]
pub struct BatchIngestionResult {
    /// Events successfully normalized, deduplicated, audited, and dispatched.
    pub successful: Vec<FinancialEvent>,
    /// Failed transactions paired with error explanations.
    pub failed: Vec<(Option<Uuid>, TransactionError)>,
}

/// Asynchronous pipeline for ingesting, validating, deduplicating, and dispatching financial transactions.
pub struct IngestionPipeline {
    deduplicator: Arc<dyn Deduplicator>,
    event_bus: Arc<dyn EventBus>,
    audit_log: Arc<dyn AuditLog>,
    filters: Vec<Arc<dyn TransactionFilter>>,
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
            filters: Vec::new(),
        }
    }

    /// Registers a custom transaction filter.
    #[must_use]
    pub fn with_filter(mut self, filter: Arc<dyn TransactionFilter>) -> Self {
        self.filters.push(filter);
        self
    }

    /// Ingests a raw transaction, executing normalization, deduplication, audit logging, and bus dispatch.
    pub async fn ingest(&self, raw: RawTransaction) -> Result<FinancialEvent, TransactionError> {
        // Step 1: Normalize and validate raw transaction payload
        let event = normalize_transaction(raw)?;

        // Step 2: Enforce deduplication / idempotency
        self.deduplicator.check_and_record(&event.event_id())?;

        // Step 3: Evaluate filters
        for filter in &self.filters {
            if !filter.should_process(&event) {
                return Ok(event);
            }
        }

        // Step 4: Record ingestion audit record
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

        // Step 5: Dispatch to the event bus for downstream processing
        self.event_bus
            .publish(&event)
            .await
            .map_err(|e| TransactionError::Bus(e.to_string()))?;

        Ok(event)
    }

    /// Ingests a batch of transactions asynchronously, collecting successes and failures.
    pub async fn ingest_batch(&self, raw_batch: Vec<RawTransaction>) -> BatchIngestionResult {
        let mut result = BatchIngestionResult::default();

        for raw in raw_batch {
            let event_id = raw.event_id;
            match self.ingest(raw).await {
                Ok(ev) => result.successful.push(ev),
                Err(err) => result.failed.push((event_id, err)),
            }
        }

        result
    }
}
