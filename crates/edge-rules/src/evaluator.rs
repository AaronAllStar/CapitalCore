//! Deterministic AST condition evaluation.

use crate::ast::Condition;
use crate::context::EvaluationContext;

/// Evaluates a condition AST node against the given evaluation context.
#[must_use]
pub fn evaluate_condition(cond: &Condition, ctx: &EvaluationContext) -> bool {
    match cond {
        Condition::AmountGreaterThan {
            minor_units,
            currency,
        } => {
            ctx.event.money().currency() == *currency
                && ctx.event.money().minor_units() > *minor_units
        }
        Condition::AmountLessThan {
            minor_units,
            currency,
        } => {
            ctx.event.money().currency() == *currency
                && ctx.event.money().minor_units() < *minor_units
        }
        Condition::EventTypeEquals(expected_type) => ctx.event.event_type() == *expected_type,
        Condition::ChannelEquals(expected_channel) => ctx.event.channel() == *expected_channel,
        Condition::MetadataEquals { key, value } => ctx
            .event
            .metadata()
            .get(key)
            .is_some_and(|actual| actual == value),
        Condition::FeatureThreshold { feature, op, value } => {
            if let Some(actual) = ctx.features.get(feature) {
                op.evaluate(*actual, *value)
            } else {
                false
            }
        }
        Condition::And(conditions) => conditions.iter().all(|c| evaluate_condition(c, ctx)),
        Condition::Or(conditions) => conditions.iter().any(|c| evaluate_condition(c, ctx)),
        Condition::Not(sub) => !evaluate_condition(sub, ctx),
        Condition::AlwaysTrue => true,
    }
}
