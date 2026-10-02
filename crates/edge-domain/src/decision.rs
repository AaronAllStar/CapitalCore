use crate::enums::RiskLevel;
use crate::id::{ModelId, RuleId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Core policy/intelligence decision outcome.
/// Per AGENTS.md invariant: Exactly 4 variants (ALLOW | REVIEW | BLOCK | ESCALATE).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Decision {
    Allow,
    Review,
    Block,
    Escalate,
}

impl Decision {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "ALLOW",
            Self::Review => "REVIEW",
            Self::Block => "BLOCK",
            Self::Escalate => "ESCALATE",
        }
    }

    /// Returns true if this decision permits transaction completion without intervention.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Returns true if transaction is outright blocked.
    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        matches!(self, Self::Block)
    }
}

impl fmt::Display for Decision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Comprehensive outcome of an intelligence evaluation pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionOutcome {
    decision: Decision,
    risk_level: RiskLevel,
    reasons: Vec<String>,
    triggered_rules: Vec<RuleId>,
    model_id: Option<ModelId>,
    evaluated_at: DateTime<Utc>,
}

impl DecisionOutcome {
    /// Constructs a new `DecisionOutcome`.
    #[must_use]
    pub fn new(
        decision: Decision,
        risk_level: RiskLevel,
        reasons: Vec<String>,
        triggered_rules: Vec<RuleId>,
        model_id: Option<ModelId>,
        evaluated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            decision,
            risk_level,
            reasons,
            triggered_rules,
            model_id,
            evaluated_at,
        }
    }

    #[must_use]
    pub const fn decision(&self) -> Decision {
        self.decision
    }

    #[must_use]
    pub const fn risk_level(&self) -> RiskLevel {
        self.risk_level
    }

    #[must_use]
    pub fn reasons(&self) -> &[String] {
        &self.reasons
    }

    #[must_use]
    pub fn triggered_rules(&self) -> &[RuleId] {
        &self.triggered_rules
    }

    #[must_use]
    pub const fn model_id(&self) -> Option<ModelId> {
        self.model_id
    }

    #[must_use]
    pub const fn evaluated_at(&self) -> DateTime<Utc> {
        self.evaluated_at
    }
}
