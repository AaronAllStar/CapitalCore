//! Ingestion deduplication and replay prevention.

use crate::error::TransactionError;
use edge_domain::EventId;
use std::collections::HashSet;
use std::sync::RwLock;

/// Interface for idempotency and duplicate event detection.
pub trait Deduplicator: Send + Sync {
    /// Verifies if an event ID has already been processed, recording it if unseen.
    fn check_and_record(&self, event_id: &EventId) -> Result<(), TransactionError>;
}

/// Thread-safe in-memory deduplication cache.
#[derive(Default)]
pub struct InMemoryDeduplicator {
    seen: RwLock<HashSet<EventId>>,
}

impl InMemoryDeduplicator {
    /// Creates a new empty `InMemoryDeduplicator`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            seen: RwLock::new(HashSet::new()),
        }
    }
}

impl Deduplicator for InMemoryDeduplicator {
    fn check_and_record(&self, event_id: &EventId) -> Result<(), TransactionError> {
        let mut seen = self
            .seen
            .write()
            .map_err(|e| TransactionError::Normalization(format!("lock error: {e}")))?;

        if seen.contains(event_id) {
            return Err(TransactionError::Duplicate(format!(
                "event with id {event_id} has already been processed"
            )));
        }

        seen.insert(*event_id);
        Ok(())
    }
}
