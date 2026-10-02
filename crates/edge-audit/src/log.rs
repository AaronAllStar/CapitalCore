use crate::error::AuditError;
use chrono::{DateTime, Utc};
use edge_domain::{AuditEvent, AuditId, EventId};
use std::future::Future;
use std::pin::Pin;

/// Type alias for heap-allocated async futures without external runtime dependencies.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait defining an append-only, tamper-evident audit storage engine.
///
/// Invariants enforced:
/// - Strictly append-only: ZERO mutation (`&mut self`) methods modifying existing records.
/// - Every decision reconstructable via immutable AuditEvents.
pub trait AuditLog: Send + Sync {
    /// Appends an `AuditEvent` to the immutable audit log.
    fn append<'a>(&'a self, event: AuditEvent) -> BoxFuture<'a, Result<AuditId, AuditError>>;

    /// Retrieves an audit event by its unique `AuditId`.
    fn get_by_audit_id<'a>(
        &'a self,
        id: AuditId,
    ) -> BoxFuture<'a, Result<Option<AuditEvent>, AuditError>>;

    /// Retrieves all audit events recorded for a given `EventId`.
    fn get_by_event_id<'a>(
        &'a self,
        event_id: EventId,
    ) -> BoxFuture<'a, Result<Vec<AuditEvent>, AuditError>>;

    /// Queries audit events occurring within a closed timestamp interval `[start, end]`.
    fn query_range<'a>(
        &'a self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<Vec<AuditEvent>, AuditError>>;

    /// Verifies the cryptographic tamper-evident hash chain from genesis to tail.
    fn verify_integrity<'a>(&'a self) -> BoxFuture<'a, Result<bool, AuditError>>;
}
