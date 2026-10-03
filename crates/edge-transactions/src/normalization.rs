//! Raw transaction payload normalization and validation into canonical financial events.

use crate::error::TransactionError;
use chrono::{DateTime, Utc};
use edge_core::{Currency, Money};
use edge_domain::{Channel, EventId, EventType, FinancialEvent, TransactionId, UserId};
use edge_security::sanitize_control_chars;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use uuid::Uuid;

/// Unsanitized incoming raw transaction payload from API or partner gateways.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawTransaction {
    /// Optional external event UUID; generated if absent.
    pub event_id: Option<Uuid>,
    /// Underlying business transaction identifier.
    pub transaction_id: Uuid,
    /// Originating customer or account UUID.
    pub user_id: Uuid,
    /// Event type string (e.g. "payment", "transfer").
    pub event_type: String,
    /// Originating channel string (e.g. "web", "mobile").
    pub channel: String,
    /// Monotonic amount in minor units (cents, satoshis).
    pub amount_minor: i64,
    /// Currency code string (e.g. "USD", "EUR").
    pub currency: String,
    /// Optional event creation timestamp; defaults to current time if omitted.
    pub timestamp: Option<DateTime<Utc>>,
    /// Free-form metadata attributes.
    pub metadata: Option<BTreeMap<String, String>>,
}

/// Normalizes and validates a raw transaction into an immutable `FinancialEvent`.
pub fn normalize_transaction(raw: RawTransaction) -> Result<FinancialEvent, TransactionError> {
    if raw.amount_minor <= 0 {
        return Err(TransactionError::Normalization(format!(
            "transaction amount must be positive, was {}",
            raw.amount_minor
        )));
    }

    let currency = Currency::from_str(&raw.currency)
        .map_err(|e| TransactionError::Normalization(e.to_string()))?;

    let money = Money::new(raw.amount_minor, currency);

    let event_type = match raw.event_type.trim().to_ascii_uppercase().as_str() {
        "PAYMENT" => EventType::Payment,
        "TRANSFER" => EventType::Transfer,
        "WITHDRAWAL" => EventType::Withdrawal,
        "DEPOSIT" => EventType::Deposit,
        "REFUND" => EventType::Refund,
        "CHARGEBACK" => EventType::Chargeback,
        "AUTHORIZATION" => EventType::Authorization,
        other => {
            return Err(TransactionError::Normalization(format!(
                "unknown event type: '{other}'"
            )))
        }
    };

    let channel = match raw.channel.trim().to_ascii_uppercase().as_str() {
        "WEB" => Channel::Web,
        "MOBILE" => Channel::Mobile,
        "API" => Channel::Api,
        "POS" => Channel::Pos,
        "ATM" => Channel::Atm,
        "BATCH" => Channel::Batch,
        other => {
            return Err(TransactionError::Normalization(format!(
                "unknown channel: '{other}'"
            )))
        }
    };

    let event_id = EventId::from_uuid(raw.event_id.unwrap_or_else(Uuid::new_v4));
    let transaction_id = TransactionId::from_uuid(raw.transaction_id);
    let user_id = UserId::from_uuid(raw.user_id);
    let created_at = raw.timestamp.unwrap_or_else(Utc::now);

    let mut sanitized_metadata = BTreeMap::new();
    if let Some(meta) = raw.metadata {
        for (k, v) in meta {
            sanitized_metadata.insert(sanitize_control_chars(&k), sanitize_control_chars(&v));
        }
    }

    Ok(FinancialEvent::new(
        event_id,
        transaction_id,
        user_id,
        event_type,
        channel,
        money,
        created_at,
        sanitized_metadata,
    ))
}
