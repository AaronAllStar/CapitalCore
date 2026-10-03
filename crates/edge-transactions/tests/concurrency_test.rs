use edge_audit::InMemoryAuditLog;
use edge_domain::FinancialEvent;
use edge_events::{BoxFuture, EventHandler, InMemoryEventBusBuilder};
use edge_transactions::{InMemoryDeduplicator, IngestionPipeline, RawTransaction};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use uuid::Uuid;

struct CountingHandler {
    counter: Arc<AtomicUsize>,
}

impl EventHandler for CountingHandler {
    fn name(&self) -> &'static str {
        "counting_handler"
    }

    fn handle<'a>(
        &'a self,
        _event: &'a FinancialEvent,
    ) -> BoxFuture<'a, Result<(), edge_events::EventError>> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}

#[tokio::test]
async fn test_concurrent_ingestion_100_transactions() {
    let counter = Arc::new(AtomicUsize::new(0));
    let bus = InMemoryEventBusBuilder::new()
        .register(Arc::new(CountingHandler {
            counter: counter.clone(),
        }))
        .build();

    let audit = Arc::new(InMemoryAuditLog::new());
    let dedup = Arc::new(InMemoryDeduplicator::new());
    let pipeline = Arc::new(IngestionPipeline::new(dedup, Arc::new(bus), audit.clone()));

    let mut handles = Vec::new();
    for _ in 0..10 {
        let pipeline_clone = pipeline.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..10 {
                let raw = RawTransaction {
                    event_id: Some(Uuid::new_v4()),
                    transaction_id: Uuid::new_v4(),
                    user_id: Uuid::new_v4(),
                    event_type: "payment".to_string(),
                    channel: "mobile".to_string(),
                    amount_minor: 500,
                    currency: "USD".to_string(),
                    timestamp: None,
                    metadata: None,
                };
                pipeline_clone.ingest(raw).await.expect("ingest");
            }
        }));
    }

    for h in handles {
        h.await.expect("task completes");
    }

    assert_eq!(counter.load(Ordering::SeqCst), 100);
    assert_eq!(audit.count(), 100);
}
