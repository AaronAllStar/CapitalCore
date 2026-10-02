use crate::error::AuditError;
use crate::log::{AuditLog, BoxFuture};
use crate::record::AuditRecord;
use chrono::{DateTime, Utc};
use edge_domain::{AuditEvent, AuditId, EventId};
use std::sync::RwLock;

const GENESIS_HASH: &str = "0000000000000000";

/// In-memory, append-only implementation of `AuditLog` featuring tamper-evident hash chaining.
#[derive(Default)]
pub struct InMemoryAuditLog {
    records: RwLock<Vec<AuditRecord>>,
}

impl InMemoryAuditLog {
    /// Constructs an empty `InMemoryAuditLog`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: RwLock::new(Vec::new()),
        }
    }

    /// Number of records currently committed in the audit log.
    #[must_use]
    pub fn count(&self) -> usize {
        self.records.read().map(|r| r.len()).unwrap_or(0)
    }
}

impl AuditLog for InMemoryAuditLog {
    fn append<'a>(&'a self, event: AuditEvent) -> BoxFuture<'a, Result<AuditId, AuditError>> {
        Box::pin(async move {
            let mut records = self.records.write().map_err(|e| {
                AuditError::StorageFailed(format!("failed to acquire write lock: {e}"))
            })?;

            let sequence = (records.len() as u64) + 1;
            let previous_hash = match records.last() {
                Some(prev) => prev.record_hash().to_string(),
                None => GENESIS_HASH.to_string(),
            };

            let audit_id = event.audit_id();
            let record = AuditRecord::new(sequence, event, previous_hash);
            records.push(record);

            Ok(audit_id)
        })
    }

    fn get_by_audit_id<'a>(
        &'a self,
        id: AuditId,
    ) -> BoxFuture<'a, Result<Option<AuditEvent>, AuditError>> {
        Box::pin(async move {
            let records = self.records.read().map_err(|e| {
                AuditError::StorageFailed(format!("failed to acquire read lock: {e}"))
            })?;

            for record in records.iter() {
                if record.event().audit_id() == id {
                    return Ok(Some(record.event().clone()));
                }
            }

            Ok(None)
        })
    }

    fn get_by_event_id<'a>(
        &'a self,
        event_id: EventId,
    ) -> BoxFuture<'a, Result<Vec<AuditEvent>, AuditError>> {
        Box::pin(async move {
            let records = self.records.read().map_err(|e| {
                AuditError::StorageFailed(format!("failed to acquire read lock: {e}"))
            })?;

            let matching: Vec<AuditEvent> = records
                .iter()
                .filter(|r| r.event().event_id() == event_id)
                .map(|r| r.event().clone())
                .collect();

            Ok(matching)
        })
    }

    fn query_range<'a>(
        &'a self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<Vec<AuditEvent>, AuditError>> {
        Box::pin(async move {
            let records = self.records.read().map_err(|e| {
                AuditError::StorageFailed(format!("failed to acquire read lock: {e}"))
            })?;

            let in_range: Vec<AuditEvent> = records
                .iter()
                .filter(|r| r.event().created_at() >= start && r.event().created_at() <= end)
                .map(|r| r.event().clone())
                .collect();

            Ok(in_range)
        })
    }

    fn verify_integrity<'a>(&'a self) -> BoxFuture<'a, Result<bool, AuditError>> {
        Box::pin(async move {
            let records = self.records.read().map_err(|e| {
                AuditError::StorageFailed(format!("failed to acquire read lock: {e}"))
            })?;

            let mut expected_prev = GENESIS_HASH.to_string();

            for (idx, record) in records.iter().enumerate() {
                let seq = (idx as u64) + 1;
                if record.sequence() != seq {
                    return Err(AuditError::IntegrityViolation {
                        sequence: seq,
                        expected: format!("seq {seq}"),
                        computed: format!("seq {}", record.sequence()),
                    });
                }

                if record.previous_hash() != expected_prev {
                    return Err(AuditError::IntegrityViolation {
                        sequence: seq,
                        expected: expected_prev,
                        computed: record.previous_hash().to_string(),
                    });
                }

                let computed_hash =
                    AuditRecord::calculate_hash(seq, &expected_prev, record.event());
                if record.record_hash() != computed_hash {
                    return Err(AuditError::IntegrityViolation {
                        sequence: seq,
                        expected: record.record_hash().to_string(),
                        computed: computed_hash,
                    });
                }

                expected_prev = record.record_hash().to_string();
            }

            Ok(true)
        })
    }
}
