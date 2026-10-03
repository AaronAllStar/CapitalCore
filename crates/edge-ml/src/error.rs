//! Error definitions for the machine learning inference engine.

use thiserror::Error;

/// Errors arising during model loading, manifest parsing, or inference execution.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum MlError {
    /// Invalid model architecture format or deserialization failure.
    #[error("failed to parse model artifact: {0}")]
    InvalidModel(String),

    /// Missing expected feature during inference.
    #[error("missing required feature '{0}'")]
    MissingFeature(String),

    /// Score computation out of bounds.
    #[error("inference evaluation produced out-of-bounds score: {0}")]
    EvaluationError(String),
}
