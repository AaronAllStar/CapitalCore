//! EdgeArena background worker subsystem.
//!
//! Provides asynchronous task queues, exponential backoff retries,
//! dead-letter queue (DLQ) isolation, and worker supervision.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod dlq;
pub mod error;
pub mod queue;
pub mod retry;
pub mod worker;

pub use dlq::{DeadLetterItem, DeadLetterQueue, InMemoryDeadLetterQueue};
pub use error::WorkerError;
pub use queue::{
    InMemoryWorkerQueue, QueueConsumer, QueueProducer, WorkerMessage, WorkerQueueProducer,
};
pub use retry::RetryPolicy;
pub use worker::{EventWorker, TaskHandler, WorkerStats};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use edge_core::{Currency, Money};
    use edge_domain::{Channel, EventId, EventType, FinancialEvent, TransactionId, UserId};
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::watch;

    fn make_test_event() -> FinancialEvent {
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

    struct MockTaskHandler {
        name: &'static str,
        fail_count: AtomicUsize,
        poison: bool,
    }

    impl MockTaskHandler {
        fn always_succeed() -> Self {
            Self {
                name: "mock_success",
                fail_count: AtomicUsize::new(0),
                poison: false,
            }
        }

        fn fail_times(times: usize) -> Self {
            Self {
                name: "mock_retry",
                fail_count: AtomicUsize::new(times),
                poison: false,
            }
        }

        fn poison() -> Self {
            Self {
                name: "mock_poison",
                fail_count: AtomicUsize::new(0),
                poison: true,
            }
        }
    }

    impl TaskHandler for MockTaskHandler {
        fn name(&self) -> &'static str {
            self.name
        }

        fn execute<'a>(
            &'a self,
            _event: &'a FinancialEvent,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorkerError>> + Send + 'a>>
        {
            Box::pin(async move {
                if self.poison {
                    return Err(WorkerError::PoisonMessage("corrupted payload".to_string()));
                }
                let current = self.fail_count.load(Ordering::SeqCst);
                if current > 0 {
                    self.fail_count.fetch_sub(1, Ordering::SeqCst);
                    return Err(WorkerError::Transient("network timeout".to_string()));
                }
                Ok(())
            })
        }
    }

    #[test]
    fn test_retry_policy_calculate_delay() {
        let policy = RetryPolicy::new(
            5,
            Duration::from_millis(100),
            Duration::from_millis(1000),
            2,
        );

        assert_eq!(policy.calculate_delay(0), Duration::ZERO);
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(400));
        assert_eq!(policy.calculate_delay(4), Duration::from_millis(800));
        assert_eq!(policy.calculate_delay(5), Duration::from_millis(1000)); // Capped at max_backoff
    }

    #[tokio::test]
    async fn test_successful_processing() {
        let handler = Arc::new(MockTaskHandler::always_succeed());
        let dlq = Arc::new(InMemoryDeadLetterQueue::new());
        let policy = RetryPolicy::default();
        let worker = EventWorker::new(handler, policy, dlq.clone());

        let msg = WorkerMessage::new(make_test_event());
        let res = worker.process_message(msg).await;

        assert!(res.is_ok());
        assert_eq!(worker.stats().processed.load(Ordering::SeqCst), 1);
        assert_eq!(worker.stats().retried.load(Ordering::SeqCst), 0);
        assert_eq!(worker.stats().dead_lettered.load(Ordering::SeqCst), 0);
        assert_eq!(dlq.count().await, 0);
    }

    #[tokio::test]
    async fn test_transient_retry_success() {
        // Fails 2 times, then succeeds on attempt 3
        let handler = Arc::new(MockTaskHandler::fail_times(2));
        let dlq = Arc::new(InMemoryDeadLetterQueue::new());
        let policy = RetryPolicy::new(3, Duration::from_millis(1), Duration::from_millis(10), 2);
        let worker = EventWorker::new(handler, policy, dlq.clone());

        let msg = WorkerMessage::new(make_test_event());
        let res = worker.process_message(msg).await;

        assert!(res.is_ok());
        assert_eq!(worker.stats().processed.load(Ordering::SeqCst), 1);
        assert_eq!(worker.stats().retried.load(Ordering::SeqCst), 2);
        assert_eq!(worker.stats().dead_lettered.load(Ordering::SeqCst), 0);
        assert_eq!(dlq.count().await, 0);
    }

    #[tokio::test]
    async fn test_retry_exhaustion_routes_to_dlq() {
        // Fails 5 times, but max_retries is 2
        let handler = Arc::new(MockTaskHandler::fail_times(5));
        let dlq = Arc::new(InMemoryDeadLetterQueue::new());
        let policy = RetryPolicy::new(2, Duration::from_millis(1), Duration::from_millis(10), 2);
        let worker = EventWorker::new(handler, policy, dlq.clone());

        let msg = WorkerMessage::new(make_test_event());
        let res = worker.process_message(msg).await;

        assert!(res.is_err());
        assert_eq!(worker.stats().processed.load(Ordering::SeqCst), 0);
        assert_eq!(worker.stats().retried.load(Ordering::SeqCst), 1);
        assert_eq!(worker.stats().dead_lettered.load(Ordering::SeqCst), 1);

        assert_eq!(dlq.count().await, 1);
        let items = dlq.list().await;
        assert_eq!(items.len(), 1);
        assert!(!items[0].is_poison);
        assert_eq!(items[0].attempts, 2);
    }

    #[tokio::test]
    async fn test_poison_message_immediate_dlq() {
        let handler = Arc::new(MockTaskHandler::poison());
        let dlq = Arc::new(InMemoryDeadLetterQueue::new());
        let policy = RetryPolicy::new(5, Duration::from_millis(1), Duration::from_millis(10), 2);
        let worker = EventWorker::new(handler, policy, dlq.clone());

        let msg = WorkerMessage::new(make_test_event());
        let res = worker.process_message(msg).await;

        assert!(res.is_err());
        assert_eq!(worker.stats().processed.load(Ordering::SeqCst), 0);
        assert_eq!(worker.stats().retried.load(Ordering::SeqCst), 0);
        assert_eq!(worker.stats().dead_lettered.load(Ordering::SeqCst), 1);

        assert_eq!(dlq.count().await, 1);
        let items = dlq.list().await;
        assert!(items[0].is_poison);
        assert_eq!(items[0].attempts, 1);
    }

    #[tokio::test]
    async fn test_worker_run_loop_with_queue() {
        let queue = Arc::new(InMemoryWorkerQueue::new(10));
        let handler = Arc::new(MockTaskHandler::always_succeed());
        let dlq = Arc::new(InMemoryDeadLetterQueue::new());
        let policy = RetryPolicy::default();
        let worker = Arc::new(EventWorker::new(handler, policy, dlq));

        let producer = queue.producer();
        for _ in 0..5 {
            producer.send(make_test_event()).await.unwrap();
        }

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let worker_clone = worker.clone();
        let queue_clone = queue.clone();

        let handle = tokio::spawn(async move {
            worker_clone.run_loop(queue_clone, shutdown_rx).await;
        });

        // Give worker time to process 5 messages
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(worker.stats().processed.load(Ordering::SeqCst), 5);

        // Signal shutdown
        shutdown_tx.send(true).unwrap();
        handle.await.unwrap();
    }
}
