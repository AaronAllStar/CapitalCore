//! # `edge-events`
//!
//! Event bus traits and in-memory implementation for the EdgeArena intelligence pipeline.
//!
//! Invariants:
//! - Pure domain crate: zero SQL, HTTP, or direct network dependencies.
//! - Handlers configured deterministically at startup.
//! - Full error isolation across handlers during event dispatch.

#![deny(unsafe_code)]

pub mod bus;
pub mod error;
pub mod handler;
pub mod in_memory;

pub use bus::EventBus;
pub use error::EventError;
pub use handler::{BoxFuture, EventHandler};
pub use in_memory::{InMemoryEventBus, InMemoryEventBusBuilder};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use edge_core::{Currency, Money};
    use edge_domain::{Channel, EventId, EventType, FinancialEvent, TransactionId, UserId};
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct CountingHandler {
        name: &'static str,
        counter: Arc<AtomicUsize>,
    }

    impl EventHandler for CountingHandler {
        fn name(&self) -> &str {
            self.name
        }

        fn handle<'a>(
            &'a self,
            _event: &'a FinancialEvent,
        ) -> BoxFuture<'a, Result<(), EventError>> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(()) })
        }
    }

    struct FailingHandler {
        name: &'static str,
        counter: Arc<AtomicUsize>,
    }

    impl EventHandler for FailingHandler {
        fn name(&self) -> &str {
            self.name
        }

        fn handle<'a>(
            &'a self,
            _event: &'a FinancialEvent,
        ) -> BoxFuture<'a, Result<(), EventError>> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Box::pin(async {
                Err(EventError::HandlerFailed {
                    handler_name: "FailingHandler".to_string(),
                    reason: "Intentional test failure".to_string(),
                })
            })
        }
    }

    fn sample_event() -> FinancialEvent {
        FinancialEvent::new(
            EventId::new(),
            TransactionId::new(),
            UserId::new(),
            EventType::Payment,
            Channel::Web,
            Money::new(1000, Currency::USD),
            Utc::now(),
            BTreeMap::new(),
        )
    }

    #[tokio::test]
    async fn test_publish_calls_all_registered_handlers() {
        let count1 = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::new(AtomicUsize::new(0));

        let h1 = Arc::new(CountingHandler {
            name: "H1",
            counter: Arc::clone(&count1),
        });
        let h2 = Arc::new(CountingHandler {
            name: "H2",
            counter: Arc::clone(&count2),
        });

        let bus = InMemoryEventBus::builder()
            .register(h1)
            .register(h2)
            .build();

        assert_eq!(bus.handler_count(), 2);

        let event = sample_event();
        let res = bus.publish(&event).await;
        assert!(res.is_ok());

        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_handler_error_isolation() {
        let count_fail = Arc::new(AtomicUsize::new(0));
        let count_succ = Arc::new(AtomicUsize::new(0));

        let h_fail = Arc::new(FailingHandler {
            name: "Failing",
            counter: Arc::clone(&count_fail),
        });
        let h_succ = Arc::new(CountingHandler {
            name: "Success",
            counter: Arc::clone(&count_succ),
        });

        let bus = InMemoryEventBus::builder()
            .register(h_fail)
            .register(h_succ)
            .build();

        let event = sample_event();
        let res = bus.publish(&event).await;

        // Failing handler returned an error, but the subsequent handler still executed
        assert!(res.is_err());
        assert_eq!(count_fail.load(Ordering::SeqCst), 1);
        assert_eq!(count_succ.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_concurrent_publish_100_events_10_handlers() {
        const HANDLER_COUNT: usize = 10;
        const EVENT_COUNT: usize = 100;

        let mut counters = Vec::new();
        let mut builder = InMemoryEventBus::builder();

        for _ in 0..HANDLER_COUNT {
            let counter = Arc::new(AtomicUsize::new(0));
            counters.push(Arc::clone(&counter));
            let handler = Arc::new(CountingHandler {
                name: "ConcurrentHandler",
                counter,
            });
            builder = builder.register(handler);
        }

        let bus = Arc::new(builder.build());
        let mut tasks = Vec::new();

        for _ in 0..EVENT_COUNT {
            let bus_clone = Arc::clone(&bus);
            tasks.push(tokio::spawn(async move {
                let event = sample_event();
                bus_clone.publish(&event).await
            }));
        }

        for task in tasks {
            let res = task.await.expect("join handle");
            assert!(res.is_ok());
        }

        // Each of the 10 handlers must have received all 100 events
        for counter in counters {
            assert_eq!(counter.load(Ordering::SeqCst), EVENT_COUNT);
        }
    }
}
