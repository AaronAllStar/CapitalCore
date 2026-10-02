use crate::error::EventError;
use edge_domain::FinancialEvent;
use std::future::Future;
use std::pin::Pin;

/// Type alias for pinned heap-allocated async futures without external runtime dependencies.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait implemented by event consumers and pipeline stages.
pub trait EventHandler: Send + Sync {
    /// Identifier for observability and error reporting.
    fn name(&self) -> &str;

    /// Dispatches an event to the handler asynchronously.
    fn handle<'a>(&'a self, event: &'a FinancialEvent) -> BoxFuture<'a, Result<(), EventError>>;
}
