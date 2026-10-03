//! Dead-letter queue (DLQ) for unprocessable or persistently failing events.

use crate::error::WorkerError;
use chrono::{DateTime, Utc};
use edge_domain::FinancialEvent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Record encapsulating an unrecoverable or failed financial event routed to the DLQ.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeadLetterItem {
    /// Unique identifier for this dead-letter entry.
    pub id: Uuid,
    /// Underlying financial event that failed processing.
    pub event: FinancialEvent,
    /// Total number of execution attempts before exhaustion.
    pub attempts: u32,
    /// Description of the final error triggering DLQ routing.
    pub last_error: String,
    /// Whether the failure was caused by a poison message.
    pub is_poison: bool,
    /// Timestamp when item was enqueued into DLQ.
    pub enqueued_at: DateTime<Utc>,
}

impl DeadLetterItem {
    /// Constructs a new `DeadLetterItem`.
    #[must_use]
    pub fn new(event: FinancialEvent, attempts: u32, last_error: String, is_poison: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            attempts,
            last_error,
            is_poison,
            enqueued_at: Utc::now(),
        }
    }
}

/// Trait defining storage and retrieval of dead-letter messages.
pub trait DeadLetterQueue: Send + Sync {
    /// Enqueues a failed event item into the DLQ.
    fn push(
        &self,
        item: DeadLetterItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorkerError>> + Send + '_>>;

    /// Retrieves all items currently stored in the DLQ.
    fn list(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<DeadLetterItem>> + Send + '_>>;

    /// Returns the total count of items residing in the DLQ.
    fn count(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = usize> + Send + '_>>;

    /// Clears all messages from the DLQ.
    fn clear(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>>;
}

/// Thread-safe in-memory DLQ store.
#[derive(Debug, Default, Clone)]
pub struct InMemoryDeadLetterQueue {
    items: Arc<RwLock<Vec<DeadLetterItem>>>,
}

impl InMemoryDeadLetterQueue {
    /// Constructs a new empty `InMemoryDeadLetterQueue`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl DeadLetterQueue for InMemoryDeadLetterQueue {
    fn push(
        &self,
        item: DeadLetterItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorkerError>> + Send + '_>>
    {
        Box::pin(async move {
            let mut lock = self.items.write().await;
            lock.push(item);
            Ok(())
        })
    }

    fn list(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<DeadLetterItem>> + Send + '_>> {
        Box::pin(async move {
            let lock = self.items.read().await;
            lock.clone()
        })
    }

    fn count(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = usize> + Send + '_>> {
        Box::pin(async move {
            let lock = self.items.read().await;
            lock.len()
        })
    }

    fn clear(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let mut lock = self.items.write().await;
            lock.clear();
        })
    }
}
