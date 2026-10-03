//! Abstract Syntax Tree (AST) definitions for deterministic fraud and compliance rules.

use edge_core::Currency;
use edge_domain::{Channel, Decision, EventType, RuleId};
use serde::{Deserialize, Serialize};

/// Relational comparison operators for feature threshold checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operator {
    /// Greater than strictly (`>`).
    Gt,
    /// Greater than or equal (`>=`).
    Gte,
    /// Less than strictly (`<`).
    Lt,
    /// Less than or equal (`<=`).
    Lte,
    /// Strict equality (`==`).
    Eq,
    /// Strict inequality (`!=`).
    Neq,
}

impl Operator {
    /// Evaluates comparison between two signed 64-bit integer values.
    #[must_use]
    pub const fn evaluate(&self, left: i64, right: i64) -> bool {
        match self {
            Self::Gt => left > right,
            Self::Gte => left >= right,
            Self::Lt => left < right,
            Self::Lte => left <= right,
            Self::Eq => left == right,
            Self::Neq => left != right,
        }
    }
}

/// Abstract condition node evaluated against transaction contexts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// Evaluates `true` if transaction currency matches and minor units exceed threshold.
    AmountGreaterThan {
        /// Minor unit threshold.
        minor_units: i64,
        /// Expected currency.
        currency: Currency,
    },
    /// Evaluates `true` if transaction currency matches and minor units are below threshold.
    AmountLessThan {
        /// Minor unit threshold.
        minor_units: i64,
        /// Expected currency.
        currency: Currency,
    },
    /// Evaluates `true` if event type matches.
    EventTypeEquals(EventType),
    /// Evaluates `true` if channel matches.
    ChannelEquals(Channel),
    /// Evaluates `true` if event metadata contains key with exact matching value.
    MetadataEquals {
        /// Metadata key.
        key: String,
        /// Expected value string.
        value: String,
    },
    /// Evaluates `true` if calculated integer feature matches operator condition against target.
    FeatureThreshold {
        /// Feature name key.
        feature: String,
        /// Relational operator.
        op: Operator,
        /// Target integer value.
        value: i64,
    },
    /// Conjunction: all sub-conditions must evaluate `true`.
    And(Vec<Condition>),
    /// Disjunction: at least one sub-condition must evaluate `true`.
    Or(Vec<Condition>),
    /// Negation: inverts truth value of sub-condition.
    Not(Box<Condition>),
    /// Unconditional match (`true`).
    AlwaysTrue,
}

/// Action produced when a rule condition is satisfied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleAction {
    /// Emits a deterministic platform decision outcome.
    YieldDecision(Decision),
}

/// Standalone declarative rule definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier for this rule.
    pub id: RuleId,
    /// Human-readable rule title.
    pub name: String,
    /// Deterministic evaluation priority (ascending: lower numerical values evaluate first).
    pub priority: u32,
    /// Logical condition predicate.
    pub condition: Condition,
    /// Outcome action executed upon matching condition.
    pub action: RuleAction,
    /// Auditable machine-readable explanation reason code.
    pub reason_code: String,
}

impl Rule {
    /// Constructs a new rule.
    #[must_use]
    pub fn new(
        id: RuleId,
        name: impl Into<String>,
        priority: u32,
        condition: Condition,
        action: RuleAction,
        reason_code: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            priority,
            condition,
            action,
            reason_code: reason_code.into(),
        }
    }
}
