//! EdgeArena transaction engine: validation, normalization, deduplication, and ingestion pipeline.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod dedup;
pub mod error;
pub mod normalization;
pub mod pipeline;

pub use dedup::{Deduplicator, InMemoryDeduplicator};
pub use error::TransactionError;
pub use normalization::{normalize_transaction, RawTransaction};
pub use pipeline::{BatchIngestionResult, IngestionPipeline, TransactionFilter};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use edge_audit::InMemoryAuditLog;
    use edge_core::Currency;
    use edge_domain::{Channel, EventId, EventType, FinancialEvent};
    use edge_events::{BoxFuture, EventHandler, InMemoryEventBusBuilder};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use uuid::Uuid;

    struct MockHandler {
        received_count: Arc<AtomicUsize>,
    }

    impl EventHandler for MockHandler {
        fn name(&self) -> &'static str {
            "mock_ingestion_handler"
        }

        fn handle<'a>(
            &'a self,
            _event: &'a FinancialEvent,
        ) -> BoxFuture<'a, Result<(), edge_events::EventError>> {
            self.received_count.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(()) })
        }
    }

    #[test]
    fn test_normalize_valid_transaction() {
        let raw = RawTransaction {
            event_id: None,
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "payment".to_string(),
            channel: "mobile".to_string(),
            amount_minor: 4999,
            currency: "USD".to_string(),
            timestamp: Some(Utc::now()),
            metadata: None,
        };

        let event = normalize_transaction(raw).expect("normalization should succeed");
        assert_eq!(event.money().currency(), Currency::USD);
        assert_eq!(event.money().minor_units(), 4999);
        assert_eq!(event.event_type(), EventType::Payment);
        assert_eq!(event.channel(), Channel::Mobile);
    }

    #[test]
    fn test_normalize_negative_amount_rejected() {
        let raw = RawTransaction {
            event_id: None,
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "payment".to_string(),
            channel: "web".to_string(),
            amount_minor: -100,
            currency: "USD".to_string(),
            timestamp: None,
            metadata: None,
        };

        let err = normalize_transaction(raw).unwrap_err();
        match err {
            TransactionError::Normalization(msg) => assert!(msg.contains("must be positive")),
            other => panic!("expected Normalization error, got {other:?}"),
        }
    }

    #[test]
    fn test_deduplicator_rejects_duplicate() {
        let dedup = InMemoryDeduplicator::new();
        let event_id = EventId::new();

        assert!(dedup.check_and_record(&event_id).is_ok());
        let err = dedup.check_and_record(&event_id).unwrap_err();
        match err {
            TransactionError::Duplicate(msg) => assert!(msg.contains("already been processed")),
            other => panic!("expected Duplicate error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_ingestion_pipeline_end_to_end() {
        let received = Arc::new(AtomicUsize::new(0));
        let bus = InMemoryEventBusBuilder::new()
            .register(Arc::new(MockHandler {
                received_count: received.clone(),
            }))
            .build();

        let audit = Arc::new(InMemoryAuditLog::new());
        let dedup = Arc::new(InMemoryDeduplicator::new());

        let pipeline = IngestionPipeline::new(dedup, Arc::new(bus), audit.clone());

        let event_uuid = Uuid::new_v4();
        let raw = RawTransaction {
            event_id: Some(event_uuid),
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "transfer".to_string(),
            channel: "api".to_string(),
            amount_minor: 125000,
            currency: "EUR".to_string(),
            timestamp: None,
            metadata: None,
        };

        let event = pipeline
            .ingest(raw.clone())
            .await
            .expect("ingestion succeeds");
        assert_eq!(event.event_id().as_uuid(), event_uuid);
        assert_eq!(received.load(Ordering::SeqCst), 1);
        assert_eq!(audit.count(), 1);

        // Attempt duplicate submission
        let dup_err = pipeline.ingest(raw).await.unwrap_err();
        match dup_err {
            TransactionError::Duplicate(_) => {}
            other => panic!("expected duplicate error, got {other:?}"),
        }
        // Audit log count and handler calls remain unchanged
        assert_eq!(received.load(Ordering::SeqCst), 1);
        assert_eq!(audit.count(), 1);
    }

    #[tokio::test]
    async fn test_batch_ingestion_partial_failures() {
        let bus = InMemoryEventBusBuilder::new().build();
        let audit = Arc::new(InMemoryAuditLog::new());
        let dedup = Arc::new(InMemoryDeduplicator::new());
        let pipeline = IngestionPipeline::new(dedup, Arc::new(bus), audit);

        let id1 = Uuid::new_v4();
        let valid_raw = RawTransaction {
            event_id: Some(id1),
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "payment".to_string(),
            channel: "web".to_string(),
            amount_minor: 1000,
            currency: "USD".to_string(),
            timestamp: None,
            metadata: None,
        };

        let invalid_raw = RawTransaction {
            event_id: None,
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "payment".to_string(),
            channel: "web".to_string(),
            amount_minor: -500, // Invalid negative amount
            currency: "USD".to_string(),
            timestamp: None,
            metadata: None,
        };

        let batch = vec![valid_raw.clone(), invalid_raw, valid_raw];
        let result = pipeline.ingest_batch(batch).await;

        assert_eq!(result.successful.len(), 1);
        assert_eq!(result.failed.len(), 2);
    }
}
