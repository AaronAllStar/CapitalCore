//! Decision arbitration strategies for resolving conflicting rule outcomes.

use edge_domain::Decision;
use edge_rules::RuleMatch;

/// Trait for arbitrating multiple matching rule decisions into a singular outcome.
pub trait ArbitrationStrategy: Send + Sync {
    /// Resolves final decision given the list of matched rules and baseline default.
    fn arbitrate(&self, matches: &[RuleMatch], default_decision: Decision) -> Decision;
}

/// Restrictive hierarchy strategy: Block > Escalate > Review > Allow.
#[derive(Debug, Clone, Copy, Default)]
pub struct MostRestrictive;

impl ArbitrationStrategy for MostRestrictive {
    fn arbitrate(&self, matches: &[RuleMatch], default_decision: Decision) -> Decision {
        let mut highest = default_decision;
        for m in matches {
            if rank(m.decision) > rank(highest) {
                highest = m.decision;
            }
        }
        highest
    }
}

/// First-match priority strategy: the rule with highest priority (evaluated first) wins.
#[derive(Debug, Clone, Copy, Default)]
pub struct PriorityFirst;

impl ArbitrationStrategy for PriorityFirst {
    fn arbitrate(&self, matches: &[RuleMatch], default_decision: Decision) -> Decision {
        matches.first().map_or(default_decision, |m| m.decision)
    }
}

const fn rank(d: Decision) -> u8 {
    match d {
        Decision::Allow => 0,
        Decision::Review => 1,
        Decision::Escalate => 2,
        Decision::Block => 3,
    }
}
