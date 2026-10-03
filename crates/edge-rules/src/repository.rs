//! Repository interface and in-memory implementation for rule storage.

use crate::ast::Rule;
use crate::error::RuleError;
use edge_domain::RuleId;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::RwLock;

/// Asynchronous boxed future for thread-safe repository operations.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait for persisting and retrieving declarative fraud rules.
pub trait RuleRepository: Send + Sync {
    /// Retrieves all rules sorted deterministically by priority and identifier.
    fn get_all<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Rule>, RuleError>>;

    /// Retrieves a single rule by its unique identifier.
    fn get_by_id<'a>(&'a self, id: RuleId) -> BoxFuture<'a, Result<Option<Rule>, RuleError>>;

    /// Inserts or updates a rule definition.
    fn save<'a>(&'a self, rule: Rule) -> BoxFuture<'a, Result<(), RuleError>>;

    /// Deletes a rule by its identifier. Returns `true` if the rule was removed.
    fn delete<'a>(&'a self, id: RuleId) -> BoxFuture<'a, Result<bool, RuleError>>;
}

/// Thread-safe in-memory implementation of `RuleRepository`.
#[derive(Default)]
pub struct InMemoryRuleRepository {
    rules: RwLock<BTreeMap<RuleId, Rule>>,
}

impl InMemoryRuleRepository {
    /// Creates a new empty `InMemoryRuleRepository`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(BTreeMap::new()),
        }
    }
}

impl RuleRepository for InMemoryRuleRepository {
    fn get_all<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Rule>, RuleError>> {
        Box::pin(async move {
            let rules = self
                .rules
                .read()
                .map_err(|e| RuleError::Storage(format!("lock error: {e}")))?;

            let mut list: Vec<Rule> = rules.values().cloned().collect();
            list.sort_by(|a, b| {
                a.priority
                    .cmp(&b.priority)
                    .then_with(|| a.id.as_uuid().cmp(&b.id.as_uuid()))
            });
            Ok(list)
        })
    }

    fn get_by_id<'a>(&'a self, id: RuleId) -> BoxFuture<'a, Result<Option<Rule>, RuleError>> {
        Box::pin(async move {
            let rules = self
                .rules
                .read()
                .map_err(|e| RuleError::Storage(format!("lock error: {e}")))?;

            Ok(rules.get(&id).cloned())
        })
    }

    fn save<'a>(&'a self, rule: Rule) -> BoxFuture<'a, Result<(), RuleError>> {
        Box::pin(async move {
            let mut rules = self
                .rules
                .write()
                .map_err(|e| RuleError::Storage(format!("lock error: {e}")))?;

            rules.insert(rule.id, rule);
            Ok(())
        })
    }

    fn delete<'a>(&'a self, id: RuleId) -> BoxFuture<'a, Result<bool, RuleError>> {
        Box::pin(async move {
            let mut rules = self
                .rules
                .write()
                .map_err(|e| RuleError::Storage(format!("lock error: {e}")))?;

            Ok(rules.remove(&id).is_some())
        })
    }
}
