//! Deterministic rule execution engine with auditable explanation tracking.

use crate::ast::{Rule, RuleAction};
use crate::context::EvaluationContext;
use crate::evaluator::evaluate_condition;
use edge_domain::{Decision, RuleId};
use serde::{Deserialize, Serialize};

/// Detailed record of a rule that matched during evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleMatch {
    /// Identifier of the triggering rule.
    pub rule_id: RuleId,
    /// Title of the triggering rule.
    pub rule_name: String,
    /// Decision outcome emitted by this rule.
    pub decision: Decision,
    /// Auditable explanation reason code.
    pub reason_code: String,
}

/// Consolidated evaluation outcome produced by the rule engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleEvaluationResult {
    /// Final platform decision resolved by priority and severity.
    pub final_decision: Decision,
    /// List of all matched rules in evaluation order.
    pub matches: Vec<RuleMatch>,
    /// Unique list of explanation codes collected from matches.
    pub reasons: Vec<String>,
}

/// Deterministic, ordered rule evaluation engine.
#[derive(Default, Debug, Clone)]
pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    /// Creates an empty `RuleEngine`.
    #[must_use]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a rule, maintaining deterministic ordering sorted by priority and rule ID.
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
        self.rules.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.id.as_uuid().cmp(&b.id.as_uuid()))
        });
    }

    /// Evaluates all rules against the provided context.
    #[must_use]
    pub fn evaluate(&self, ctx: &EvaluationContext) -> RuleEvaluationResult {
        let mut matches = Vec::new();
        let mut reasons = Vec::new();
        let mut highest_severity = Decision::Allow;

        for rule in &self.rules {
            if evaluate_condition(&rule.condition, ctx) {
                let RuleAction::YieldDecision(dec) = rule.action;
                matches.push(RuleMatch {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    decision: dec,
                    reason_code: rule.reason_code.clone(),
                });

                if !reasons.contains(&rule.reason_code) {
                    reasons.push(rule.reason_code.clone());
                }

                // Severity ranking: Block > Escalate > Review > Allow
                if severity_rank(dec) > severity_rank(highest_severity) {
                    highest_severity = dec;
                }
            }
        }

        RuleEvaluationResult {
            final_decision: highest_severity,
            matches,
            reasons,
        }
    }
}

const fn severity_rank(d: Decision) -> u8 {
    match d {
        Decision::Allow => 0,
        Decision::Review => 1,
        Decision::Escalate => 2,
        Decision::Block => 3,
    }
}
