//! Overload, queue saturation, backpressure, and graceful degradation test suite.
//!
//! Validates:
//! - Bounded queue backpressure when queue capacity is reached.
//! - Supervised worker pool performance under overload.
//! - Dead-letter queue isolation for poison messages under sustained pressure.
//! - Token bucket rate limiter deterministic degradation under burst saturation.

use chrono::Utc;
use edge_core::{Currency, Money};
use edge_domain::{Channel, EventId, EventType, FinancialEvent, TransactionId, UserId};
use edge_security::{RateLimitResult, RateLimiter, TokenBucketLimiter};
use edge_workers::{
    DeadLetterQueue, EventWorker, InMemoryDeadLetterQueue, InMemoryWorkerQueue, QueueConsumer,
    QueueProducer, RetryPolicy, TaskHandler, WorkerError, WorkerSupervisor,
};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

fn make_test_event(amount: i64) -> FinancialEvent {
    FinancialEvent::new(
        EventId::new(),
        TransactionId::new(),
        UserId::new(),
        EventType::Payment,
        Channel::Web,
        Money::new(amount, Currency::USD),
        Utc::now(),
        BTreeMap::new(),
    )
}

struct MockOverloadHandler {
    processed_count: AtomicUsize,
    poison_count: AtomicUsize,
}

impl MockOverloadHandler {
    fn new() -> Self {
        Self {
            processed_count: AtomicUsize::new(0),
            poison_count: AtomicUsize::new(0),
        }
    }
}

impl TaskHandler for MockOverloadHandler {
    fn name(&self) -> &'static str {
        "MockOverloadHandler"
    }

    fn execute<'a>(
        &'a self,
        event: &'a FinancialEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorkerError>> + Send + 'a>>
    {
        Box::pin(async move {
            if event.money().minor_units() < 0 {
                self.poison_count.fetch_add(1, Ordering::SeqCst);
                Err(WorkerError::PoisonMessage(
                    "poison event rejected".to_string(),
                ))
            } else {
                self.processed_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
    }
}

#[tokio::test]
async fn test_bounded_queue_backpressure_and_drain() {
    let capacity = 3;
    let queue = InMemoryWorkerQueue::new(capacity);
    let producer = queue.producer();

    // Fill queue up to its exact bounded capacity
    for i in 1..=capacity {
        let res = timeout(
            Duration::from_millis(100),
            producer.send(make_test_event(i as i64)),
        )
        .await;
        assert!(
            res.is_ok(),
            "Item {i} should fit in bounded queue without blocking"
        );
        res.unwrap().expect("send succeeded");
    }

    // Attempting to send (capacity + 1)th item MUST block (backpressure)
    let overflow_result = timeout(
        Duration::from_millis(50),
        producer.send(make_test_event(999)),
    )
    .await;
    assert!(
        overflow_result.is_err(),
        "Queue is saturated: send must block until capacity is released"
    );

    // Consume one item to relieve backpressure
    let received = queue
        .recv()
        .await
        .expect("recv succeeded")
        .expect("item exists");
    assert_eq!(received.event.money().minor_units(), 1);

    // Now sending should unblock and succeed immediately
    let unblocked_result = timeout(
        Duration::from_millis(100),
        producer.send(make_test_event(999)),
    )
    .await;
    assert!(unblocked_result.is_ok(), "Send unblocked after drain");
    unblocked_result.unwrap().expect("send succeeded");
}

#[tokio::test]
async fn test_overload_worker_pool_with_dlq_isolation() {
    let queue = Arc::new(InMemoryWorkerQueue::new(100));
    let handler = Arc::new(MockOverloadHandler::new());
    let dlq = Arc::new(InMemoryDeadLetterQueue::new());

    // Zero-retry policy for immediate DLQ routing on poison items
    let retry_policy = RetryPolicy::new(0, Duration::from_millis(1), Duration::from_millis(5), 2);
    let worker = Arc::new(EventWorker::new(handler.clone(), retry_policy, dlq.clone()));

    let mut supervisor = WorkerSupervisor::new(worker, queue.clone(), 4);
    supervisor.start();

    let producer = queue.producer();

    // Enqueue 40 valid events and 10 poison events (negative amounts)
    for i in 0..50 {
        let amount = if i % 5 == 0 { -100 } else { 1000 + i };
        producer.send(make_test_event(amount)).await.unwrap();
    }

    // Allow workers to drain the queue
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Gracefully shutdown workers
    supervisor
        .shutdown(Duration::from_secs(1))
        .await
        .expect("shutdown clean");

    // Verify isolation: 40 processed, 10 sent to DLQ, zero crashes
    assert_eq!(handler.processed_count.load(Ordering::SeqCst), 40);
    assert_eq!(handler.poison_count.load(Ordering::SeqCst), 10);
    assert_eq!(dlq.count().await, 10);

    let dlq_items = dlq.list().await;
    assert_eq!(dlq_items.len(), 10);
    for item in dlq_items {
        assert_eq!(item.event.money().minor_units(), -100);
        assert_eq!(item.attempts, 1);
        assert!(item.is_poison);
    }
}

#[tokio::test]
async fn test_rate_limiter_burst_overload_degradation() {
    let capacity = 5;
    let refill_rate = 1; // 1 token per second
    let limiter = TokenBucketLimiter::new(capacity, refill_rate);
    let client_key = "user_burst_test";

    let mut allowed = 0;
    let mut exceeded = 0;

    for _ in 0..15 {
        match limiter.check(client_key) {
            RateLimitResult::Allowed {
                remaining: _,
                reset_after_ms: _,
            } => {
                allowed += 1;
            }
            RateLimitResult::Exceeded { retry_after_ms } => {
                exceeded += 1;
                assert!(retry_after_ms > 0, "Retry suggestion must be positive");
            }
        }
    }

    // Exactly 5 allowed, 10 rejected gracefully
    assert_eq!(allowed, 5);
    assert_eq!(exceeded, 10);
}
