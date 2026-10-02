use edge_domain::AuditEvent;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Tamper-evident wrapper binding an `AuditEvent` to a monotonic sequence and hash chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditRecord {
    sequence: u64,
    event: AuditEvent,
    previous_hash: String,
    record_hash: String,
}

impl AuditRecord {
    /// Constructs a new sequenced record linked to the previous record hash.
    #[must_use]
    pub fn new(sequence: u64, event: AuditEvent, previous_hash: String) -> Self {
        let record_hash = Self::calculate_hash(sequence, &previous_hash, &event);
        Self {
            sequence,
            event,
            previous_hash,
            record_hash,
        }
    }

    /// Monotonic 1-based sequence number.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Immutable reference to the wrapped `AuditEvent`.
    #[must_use]
    pub const fn event(&self) -> &AuditEvent {
        &self.event
    }

    /// Hash of the prior entry in the tamper-evident chain.
    #[must_use]
    pub fn previous_hash(&self) -> &str {
        &self.previous_hash
    }

    /// Tamper-evident hash of this record.
    #[must_use]
    pub fn record_hash(&self) -> &str {
        &self.record_hash
    }

    /// Computes deterministic hash binding sequence, previous hash, and audit event fields.
    #[must_use]
    pub fn calculate_hash(sequence: u64, previous_hash: &str, event: &AuditEvent) -> String {
        let mut hasher = DefaultHasher::new();
        sequence.hash(&mut hasher);
        previous_hash.hash(&mut hasher);
        event.audit_id().as_uuid().hash(&mut hasher);
        event.event_id().as_uuid().hash(&mut hasher);
        event.decision().as_str().hash(&mut hasher);
        for reason in event.reasons() {
            reason.hash(&mut hasher);
        }
        event
            .created_at()
            .timestamp_nanos_opt()
            .unwrap_or(0)
            .hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}
