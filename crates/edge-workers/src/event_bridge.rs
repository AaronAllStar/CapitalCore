//! Bridge between `edge-events` event bus and background worker queues.

use crate::queue::QueueProducer;
use edge_domain::FinancialEvent;
use edge_events::{BoxFuture, EventHandler};
use std::sync::Arc;

/// An `EventHandler` that forwards received events into an asynchronous `QueueProducer`.
pub struct QueueingEventHandler {
    producer: Arc<dyn QueueProducer>,
    name: &'static str,
}

impl QueueingEventHandler {
    /// Constructs a new `QueueingEventHandler`.
    #[must_use]
    pub fn new(producer: Arc<dyn QueueProducer>, name: &'static str) -> Self {
        Self { producer, name }
    }
}

impl EventHandler for QueueingEventHandler {
    fn name(&self) -> &'static str {
        self.name
    }

    fn handle<'a>(
        &'a self,
        event: &'a FinancialEvent,
    ) -> BoxFuture<'a, Result<(), edge_events::EventError>> {
        let producer = self.producer.clone();
        let ev = event.clone();
        let name = self.name;
        Box::pin(async move {
            producer
                .send(ev)
                .await
                .map_err(|e| edge_events::EventError::HandlerFailed {
                    handler_name: name.to_string(),
                    reason: e.to_string(),
                })
        })
    }
}
