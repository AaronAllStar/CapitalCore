use crate::bus::EventBus;
use crate::error::EventError;
use crate::handler::{BoxFuture, EventHandler};
use edge_domain::FinancialEvent;
use std::sync::Arc;
use std::time::Duration;

/// In-memory asynchronous event bus.
///
/// Handlers are registered at startup (during build) to ensure deterministic, immutably
/// configured execution order at runtime without lock contention.
#[derive(Clone)]
pub struct InMemoryEventBus {
    handlers: Arc<Vec<Arc<dyn EventHandler>>>,
    timeout: Option<Duration>,
}

impl InMemoryEventBus {
    /// Constructs a new builder to configure handlers at startup.
    #[must_use]
    pub fn builder() -> InMemoryEventBusBuilder {
        InMemoryEventBusBuilder::new()
    }

    /// Constructs an `InMemoryEventBus` with a predefined list of handlers.
    #[must_use]
    pub fn new(handlers: Vec<Arc<dyn EventHandler>>) -> Self {
        Self {
            handlers: Arc::new(handlers),
            timeout: None,
        }
    }

    /// Constructs an `InMemoryEventBus` with handler timeout bounds.
    #[must_use]
    pub fn with_timeout(handlers: Vec<Arc<dyn EventHandler>>, timeout: Duration) -> Self {
        Self {
            handlers: Arc::new(handlers),
            timeout: Some(timeout),
        }
    }

    /// Number of registered handlers.
    #[must_use]
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    /// Configured timeout per dispatch, if specified.
    #[must_use]
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }
}

impl EventBus for InMemoryEventBus {
    fn publish<'a>(&'a self, event: &'a FinancialEvent) -> BoxFuture<'a, Result<(), EventError>> {
        Box::pin(async move {
            let mut last_err = None;

            for handler in self.handlers.iter() {
                // Invoke handler; isolate errors so one failing handler does not halt execution
                if let Err(e) = handler.handle(event).await {
                    last_err = Some(e);
                }
            }

            // If any handler failed, return the error, but all handlers have been called
            if let Some(err) = last_err {
                Err(err)
            } else {
                Ok(())
            }
        })
    }
}

/// Builder for startup configuration of `InMemoryEventBus`.
#[derive(Default)]
pub struct InMemoryEventBusBuilder {
    handlers: Vec<Arc<dyn EventHandler>>,
    timeout: Option<Duration>,
}

impl InMemoryEventBusBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            timeout: None,
        }
    }

    /// Registers an event handler at startup.
    #[must_use]
    pub fn register(mut self, handler: Arc<dyn EventHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Sets an execution timeout per event dispatch.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Finalizes the event bus instance.
    #[must_use]
    pub fn build(self) -> InMemoryEventBus {
        InMemoryEventBus {
            handlers: Arc::new(self.handlers),
            timeout: self.timeout,
        }
    }
}
