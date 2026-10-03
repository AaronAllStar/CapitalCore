//! Bounded asynchronous task queuing abstractions.

use crate::error::WorkerError;
use edge_domain::FinancialEvent;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Message submitted to a worker queue.
#[derive(Debug, Clone)]
pub struct WorkerMessage {
    /// Ingested financial event.
    pub event: FinancialEvent,
    /// Number of attempts made on this message so far.
    pub attempt: u32,
}

impl WorkerMessage {
    /// Constructs a new `WorkerMessage` initialized with zero attempts.
    #[must_use]
    pub fn new(event: FinancialEvent) -> Self {
        Self { event, attempt: 0 }
    }
}

/// Producer interface for enqueuing financial events into background worker queues.
pub trait QueueProducer: Send + Sync {
    /// Sends an event to the queue with backpressure support.
    fn send(
        &self,
        event: FinancialEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorkerError>> + Send + '_>>;
}

/// Consumer interface for receiving worker messages from background worker queues.
pub trait QueueConsumer: Send + Sync {
    /// Receives the next available message, or returns `None` if the queue is closed and empty.
    fn recv(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<WorkerMessage>, WorkerError>>
                + Send
                + '_,
        >,
    >;
}

/// In-memory bounded channel queue implementing both producer and consumer.
pub struct InMemoryWorkerQueue {
    sender: mpsc::Sender<WorkerMessage>,
    receiver: Arc<Mutex<mpsc::Receiver<WorkerMessage>>>,
}

impl InMemoryWorkerQueue {
    /// Constructs a new `InMemoryWorkerQueue` with a bounded buffer capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    /// Clones the sender to create a dedicated producer handle.
    #[must_use]
    pub fn producer(&self) -> WorkerQueueProducer {
        WorkerQueueProducer {
            sender: self.sender.clone(),
        }
    }
}

/// Lightweight cloned producer handle for `InMemoryWorkerQueue`.
#[derive(Clone)]
pub struct WorkerQueueProducer {
    sender: mpsc::Sender<WorkerMessage>,
}

impl QueueProducer for WorkerQueueProducer {
    fn send(
        &self,
        event: FinancialEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorkerError>> + Send + '_>>
    {
        let msg = WorkerMessage::new(event);
        Box::pin(async move {
            self.sender
                .send(msg)
                .await
                .map_err(|_| WorkerError::QueueClosed)
        })
    }
}

impl QueueProducer for InMemoryWorkerQueue {
    fn send(
        &self,
        event: FinancialEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorkerError>> + Send + '_>>
    {
        let msg = WorkerMessage::new(event);
        Box::pin(async move {
            self.sender
                .send(msg)
                .await
                .map_err(|_| WorkerError::QueueClosed)
        })
    }
}

impl QueueConsumer for InMemoryWorkerQueue {
    fn recv(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<WorkerMessage>, WorkerError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let mut rx = self.receiver.lock().await;
            Ok(rx.recv().await)
        })
    }
}
