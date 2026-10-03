//! Asynchronous event worker loop with backoff retry and DLQ routing.

use crate::dlq::{DeadLetterItem, DeadLetterQueue};
use crate::error::WorkerError;
use crate::queue::{QueueConsumer, WorkerMessage};
use crate::retry::RetryPolicy;
use edge_domain::FinancialEvent;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{error, info, warn};

/// Handler interface implemented by background consumers processing financial events.
pub trait TaskHandler: Send + Sync {
    /// Handler human-readable identifier.
    fn name(&self) -> &'static str;

    /// Executes the task for the given event.
    fn execute<'a>(
        &'a self,
        event: &'a FinancialEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorkerError>> + Send + 'a>>;
}

/// Statistics on worker processing lifecycle.
#[derive(Debug, Default)]
pub struct WorkerStats {
    /// Total events successfully processed.
    pub processed: AtomicUsize,
    /// Total transient retries triggered.
    pub retried: AtomicUsize,
    /// Total messages routed to the DLQ.
    pub dead_lettered: AtomicUsize,
}

/// Background worker managing handler execution, retry backoff, and DLQ dispatch.
pub struct EventWorker {
    handler: Arc<dyn TaskHandler>,
    retry_policy: RetryPolicy,
    dlq: Arc<dyn DeadLetterQueue>,
    stats: Arc<WorkerStats>,
}

impl EventWorker {
    /// Constructs a new `EventWorker`.
    #[must_use]
    pub fn new(
        handler: Arc<dyn TaskHandler>,
        retry_policy: RetryPolicy,
        dlq: Arc<dyn DeadLetterQueue>,
    ) -> Self {
        Self {
            handler,
            retry_policy,
            dlq,
            stats: Arc::new(WorkerStats::default()),
        }
    }

    /// Returns a reference to worker statistics.
    #[must_use]
    pub fn stats(&self) -> &WorkerStats {
        &self.stats
    }

    /// Processes a single message through retry evaluation and DLQ routing.
    pub async fn process_message(&self, mut message: WorkerMessage) -> Result<(), WorkerError> {
        loop {
            message.attempt += 1;
            match self.handler.execute(&message.event).await {
                Ok(()) => {
                    self.stats.processed.fetch_add(1, Ordering::SeqCst);
                    return Ok(());
                }
                Err(WorkerError::PoisonMessage(reason)) => {
                    warn!(
                        handler = self.handler.name(),
                        event_id = %message.event.event_id(),
                        reason = %reason,
                        "Poison message detected, routing immediately to DLQ"
                    );
                    let item =
                        DeadLetterItem::new(message.event, message.attempt, reason.clone(), true);
                    self.dlq.push(item).await?;
                    self.stats.dead_lettered.fetch_add(1, Ordering::SeqCst);
                    return Err(WorkerError::PoisonMessage(reason));
                }
                Err(WorkerError::Transient(reason)) => {
                    if self.retry_policy.should_retry(message.attempt) {
                        self.stats.retried.fetch_add(1, Ordering::SeqCst);
                        let delay = self.retry_policy.calculate_delay(message.attempt);
                        warn!(
                            handler = self.handler.name(),
                            event_id = %message.event.event_id(),
                            attempt = message.attempt,
                            max_retries = self.retry_policy.max_retries,
                            delay_ms = delay.as_millis(),
                            reason = %reason,
                            "Transient failure, applying exponential backoff retry"
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        error!(
                            handler = self.handler.name(),
                            event_id = %message.event.event_id(),
                            attempts = message.attempt,
                            reason = %reason,
                            "Max retries exhausted, routing event to DLQ"
                        );
                        let item = DeadLetterItem::new(
                            message.event,
                            message.attempt,
                            reason.clone(),
                            false,
                        );
                        self.dlq.push(item).await?;
                        self.stats.dead_lettered.fetch_add(1, Ordering::SeqCst);
                        return Err(WorkerError::MaxRetriesExceeded {
                            attempts: message.attempt,
                            reason,
                        });
                    }
                }
                Err(other) => {
                    error!(
                        handler = self.handler.name(),
                        error = %other,
                        "Unexpected error during task execution, routing to DLQ"
                    );
                    let item = DeadLetterItem::new(
                        message.event,
                        message.attempt,
                        other.to_string(),
                        false,
                    );
                    self.dlq.push(item).await?;
                    self.stats.dead_lettered.fetch_add(1, Ordering::SeqCst);
                    return Err(other);
                }
            }
        }
    }

    /// Runs the worker loop until shutdown signal is received or queue is drained and closed.
    pub async fn run_loop(
        &self,
        consumer: Arc<dyn QueueConsumer>,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        info!(handler = self.handler.name(), "Starting worker event loop");
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!(handler = self.handler.name(), "Worker received shutdown signal, exiting loop");
                        break;
                    }
                }
                res = consumer.recv() => {
                    match res {
                        Ok(Some(msg)) => {
                            let _ = self.process_message(msg).await;
                        }
                        Ok(None) => {
                            info!(handler = self.handler.name(), "Queue empty and closed, worker exiting");
                            break;
                        }
                        Err(err) => {
                            error!(handler = self.handler.name(), error = %err, "Error reading from queue");
                            break;
                        }
                    }
                }
            }
        }
    }
}
