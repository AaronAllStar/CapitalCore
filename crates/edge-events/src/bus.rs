use crate::error::EventError;
use crate::handler::BoxFuture;
use edge_domain::FinancialEvent;

/// Abstract asynchronous event bus trait.
pub trait EventBus: Send + Sync {
    /// Publishes a financial event across all registered handlers.
    fn publish<'a>(&'a self, event: &'a FinancialEvent) -> BoxFuture<'a, Result<(), EventError>>;
}
