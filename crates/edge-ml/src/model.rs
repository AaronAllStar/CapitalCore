//! Machine learning fraud decision tree inference model.

use crate::error::MlError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Single node in the decision tree: either a branch split or a terminal leaf.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DecisionNode {
    /// Terminal leaf yielding fraud probability score in `[0.0, 1.0]`.
    #[serde(rename = "leaf")]
    Leaf {
        /// Probability score.
        value: f64,
    },
    /// Internal branch splitting on feature threshold.
    #[serde(rename = "split")]
    Split {
        /// Feature name evaluated.
        feature: String,
        /// Numeric threshold for left branch (`<= threshold`).
        threshold: f64,
        /// Branch taken when `feature <= threshold`.
        left: Box<DecisionNode>,
        /// Branch taken when `feature > threshold`.
        right: Box<DecisionNode>,
    },
}

impl DecisionNode {
    /// Recursively traverses the tree to evaluate the sample features.
    pub fn evaluate(&self, features: &BTreeMap<String, i64>) -> Result<f64, MlError> {
        match self {
            Self::Leaf { value } => Ok(*value),
            Self::Split {
                feature,
                threshold,
                left,
                right,
            } => {
                let val = features
                    .get(feature)
                    .copied()
                    .ok_or_else(|| MlError::MissingFeature(feature.clone()))?;

                if (val as f64) <= *threshold {
                    left.evaluate(features)
                } else {
                    right.evaluate(features)
                }
            }
        }
    }
}

/// Serialized model specification containing features, threshold, and tree graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelManifest {
    /// Identifier of model algorithm (e.g. "decision_tree").
    pub model_type: String,
    /// Ordered list of feature names consumed by the model.
    pub features: Vec<String>,
    /// Threshold above which a sample is classified as fraud.
    pub decision_threshold: f64,
    /// Root node of the decision tree.
    pub architecture: DecisionNode,
}

/// Fraud detection model evaluator.
#[derive(Debug, Clone)]
pub struct FraudModel {
    manifest: ModelManifest,
}

impl FraudModel {
    /// Constructs a `FraudModel` from a pre-parsed `ModelManifest`.
    #[must_use]
    pub fn new(manifest: ModelManifest) -> Self {
        Self { manifest }
    }

    /// Loads model from a JSON manifest string.
    pub fn load_from_json(json_str: &str) -> Result<Self, MlError> {
        let manifest: ModelManifest = serde_json::from_str(json_str)
            .map_err(|e| MlError::InvalidModel(format!("failed to parse manifest JSON: {e}")))?;
        Ok(Self::new(manifest))
    }

    /// Loads model from an ONNX binary package containing standard header and architecture JSON.
    pub fn load_from_onnx_bytes(bytes: &[u8]) -> Result<Self, MlError> {
        // Find JSON payload boundary after header separator b"\n\x00"
        let separator = b"\n\x00";
        let pos = bytes
            .windows(separator.len())
            .position(|w| w == separator)
            .ok_or_else(|| MlError::InvalidModel("missing ONNX payload delimiter".to_string()))?;

        let payload = &bytes[pos + separator.len()..];
        let json_str = std::str::from_utf8(payload)
            .map_err(|e| MlError::InvalidModel(format!("invalid UTF-8 in ONNX payload: {e}")))?;

        let node: DecisionNode = serde_json::from_str(json_str)
            .map_err(|e| MlError::InvalidModel(format!("failed to parse ONNX graph: {e}")))?;

        let manifest = ModelManifest {
            model_type: "decision_tree".to_string(),
            features: vec![
                "amount_minor".to_string(),
                "user_txn_count_5m".to_string(),
                "user_txn_sum_5m".to_string(),
                "user_txn_count_1h".to_string(),
                "user_txn_sum_1h".to_string(),
                "user_txn_max_1h".to_string(),
                "user_txn_count_24h".to_string(),
                "user_txn_sum_24h".to_string(),
                "user_distinct_merchants_24h".to_string(),
            ],
            decision_threshold: 0.5,
            architecture: node,
        };

        Ok(Self::new(manifest))
    }

    /// Returns the required feature names.
    #[must_use]
    pub fn features(&self) -> &[String] {
        &self.manifest.features
    }

    /// Computes raw fraud probability score in `[0.0, 1.0]`.
    pub fn predict_score(&self, features: &BTreeMap<String, i64>) -> Result<f64, MlError> {
        let score = self.manifest.architecture.evaluate(features)?;
        if !(0.0..=1.0).contains(&score) {
            return Err(MlError::EvaluationError(format!(
                "score {score} outside [0.0, 1.0]"
            )));
        }
        Ok(score)
    }

    /// Evaluates binary classification decision against the configured threshold.
    pub fn is_fraud(&self, features: &BTreeMap<String, i64>) -> Result<bool, MlError> {
        let score = self.predict_score(features)?;
        Ok(score >= self.manifest.decision_threshold)
    }

    /// Converts fraud probability into deterministic integer basis points `[0, 10000]`.
    /// Zero floating-point representation leaks into downstream decision engine paths.
    pub fn predict_risk_bps(&self, features: &BTreeMap<String, i64>) -> Result<u32, MlError> {
        let score = self.predict_score(features)?;
        let bps = (score * 10_000.0).round() as u32;
        Ok(bps.min(10_000))
    }
}
