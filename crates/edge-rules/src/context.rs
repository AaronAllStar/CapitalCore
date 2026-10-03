//! Evaluation context providing transaction data and computed features to rules.

use edge_domain::FinancialEvent;
use std::collections::BTreeMap;

/// Evaluation context supplied to rules during assessment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationContext {
    /// Ingested financial transaction event.
    pub event: FinancialEvent,
    /// Pre-computed integer feature map (e.g. velocity counts, risk scores).
    pub features: BTreeMap<String, i64>,
}

impl EvaluationContext {
    /// Creates a new evaluation context for an event with given features.
    #[must_use]
    pub fn new(event: FinancialEvent, features: BTreeMap<String, i64>) -> Self {
        Self { event, features }
    }

    /// Creates an evaluation context with empty features.
    #[must_use]
    pub fn for_event(event: FinancialEvent) -> Self {
        Self {
            event,
            features: BTreeMap::new(),
        }
    }
}
