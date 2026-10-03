//! Worker pool supervisor and graceful shutdown coordinator.

use crate::error::WorkerError;
use crate::queue::QueueConsumer;
use crate::worker::EventWorker;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing::{info, warn};

/// Supervised worker pool managing concurrent worker threads and lifecycle coordination.
pub struct WorkerSupervisor {
    worker: Arc<EventWorker>,
    consumer: Arc<dyn QueueConsumer>,
    concurrency: usize,
    shutdown_tx: watch::Sender<bool>,
    handles: Vec<JoinHandle<()>>,
}

impl WorkerSupervisor {
    /// Constructs a new `WorkerSupervisor` ready to spawn worker tasks.
    #[must_use]
    pub fn new(
        worker: Arc<EventWorker>,
        consumer: Arc<dyn QueueConsumer>,
        concurrency: usize,
    ) -> Self {
        let (shutdown_tx, _) = watch::channel(false);
        Self {
            worker,
            consumer,
            concurrency: concurrency.max(1),
            shutdown_tx,
            handles: Vec::new(),
        }
    }

    /// Spawns the configured number of background worker tasks.
    pub fn start(&mut self) {
        info!(
            concurrency = self.concurrency,
            "Spawning worker supervisor task pool"
        );
        for _ in 0..self.concurrency {
            let worker = self.worker.clone();
            let consumer = self.consumer.clone();
            let shutdown_rx = self.shutdown_tx.subscribe();

            let handle = tokio::spawn(async move {
                worker.run_loop(consumer, shutdown_rx).await;
            });
            self.handles.push(handle);
        }
    }

    /// Returns the active number of spawned worker task handles.
    #[must_use]
    pub fn active_workers(&self) -> usize {
        self.handles.len()
    }

    /// Broadcasts a shutdown signal and waits up to `timeout` for all worker tasks to drain and exit.
    pub async fn shutdown(&mut self, timeout: Duration) -> Result<(), WorkerError> {
        info!("Initiating graceful worker shutdown");
        let _ = self.shutdown_tx.send(true);

        let wait_all = async {
            for handle in self.handles.drain(..) {
                let _ = handle.await;
            }
        };

        match tokio::time::timeout(timeout, wait_all).await {
            Ok(()) => {
                info!("All worker tasks terminated cleanly");
                Ok(())
            }
            Err(_) => {
                warn!("Worker tasks failed to terminate within shutdown timeout");
                Err(WorkerError::Transient(
                    "shutdown timeout exceeded".to_string(),
                ))
            }
        }
    }
}
