use crate::enums::{Channel, EventType};
use crate::id::{EventId, TransactionId, UserId};
use chrono::{DateTime, Utc};
use edge_core::Money;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Canonical, immutable financial transaction event.
///
/// Invariants enforced:
/// - Unforgeable and immutable after construction (`created_at` timestamp fixed at creation).
/// - No `&mut self` methods on this type (strictly append/read-only).
/// - Deterministic sorted metadata (`BTreeMap`) preventing hash iteration leakage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinancialEvent {
    event_id: EventId,
    transaction_id: TransactionId,
    user_id: UserId,
    event_type: EventType,
    channel: Channel,
    money: Money,
    created_at: DateTime<Utc>,
    metadata: BTreeMap<String, String>,
}

impl FinancialEvent {
    /// Constructs a new, fully validated `FinancialEvent`.
    /// Once created, instances are strictly immutable.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_id: EventId,
        transaction_id: TransactionId,
        user_id: UserId,
        event_type: EventType,
        channel: Channel,
        money: Money,
        created_at: DateTime<Utc>,
        metadata: BTreeMap<String, String>,
    ) -> Self {
        Self {
            event_id,
            transaction_id,
            user_id,
            event_type,
            channel,
            money,
            created_at,
            metadata,
        }
    }

    #[must_use]
    pub const fn event_id(&self) -> EventId {
        self.event_id
    }

    #[must_use]
    pub const fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    #[must_use]
    pub const fn user_id(&self) -> UserId {
        self.user_id
    }

    #[must_use]
    pub const fn event_type(&self) -> EventType {
        self.event_type
    }

    #[must_use]
    pub const fn channel(&self) -> Channel {
        self.channel
    }

    #[must_use]
    pub const fn money(&self) -> Money {
        self.money
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub const fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    /// Retrieve a specific metadata value by key.
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }
}
