//! EdgeArena machine learning inference engine.
//!
//! Evaluates trained fraud models in the Rust execution path without native C-dependency overhead.
//! Yields deterministic risk scores in basis points (`[0, 10000]`) to ensure zero float leaks into audit paths.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod model;

pub use error::MlError;
pub use model::{DecisionNode, FraudModel, ModelManifest};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::BTreeMap;

    const MANIFEST_JSON: &str = include_str!("../../../ml/models/model_manifest.json");
    const ONNX_BYTES: &[u8] = include_bytes!("../../../ml/models/fraud_detector.onnx");
    const SAMPLES_JSON: &str = include_str!("../../../ml/models/sample_inferences.json");

    #[test]
    fn test_load_model_from_manifest_json() {
        let model = FraudModel::load_from_json(MANIFEST_JSON).expect("valid manifest");
        assert_eq!(model.features().len(), 9);
    }

    #[test]
    fn test_load_model_from_onnx_bytes() {
        let model = FraudModel::load_from_onnx_bytes(ONNX_BYTES).expect("valid onnx bytes");
        assert_eq!(model.features().len(), 9);
    }

    #[test]
    fn test_parity_with_python_golden_inferences() {
        let model_json = FraudModel::load_from_json(MANIFEST_JSON).unwrap();
        let model_onnx = FraudModel::load_from_onnx_bytes(ONNX_BYTES).unwrap();

        let samples_data: Value = serde_json::from_str(SAMPLES_JSON).unwrap();
        let samples = samples_data["samples"].as_array().unwrap();

        for (idx, sample) in samples.iter().enumerate() {
            let feat_obj = sample["features"].as_object().unwrap();
            let mut features = BTreeMap::new();
            for (k, v) in feat_obj {
                features.insert(k.clone(), v.as_i64().unwrap());
            }

            let expected_score = sample["fraud_score"].as_f64().unwrap();
            let expected_pred = sample["prediction"].as_i64().unwrap() == 1;

            let score_json = model_json.predict_score(&features).unwrap();
            let score_onnx = model_onnx.predict_score(&features).unwrap();

            // Parity within floating point rounding delta (0.0001)
            assert!(
                (score_json - expected_score).abs() < 0.001,
                "Sample {idx}: JSON score {score_json} mismatch with expected {expected_score}"
            );
            assert!(
                (score_onnx - expected_score).abs() < 0.001,
                "Sample {idx}: ONNX score {score_onnx} mismatch with expected {expected_score}"
            );

            let pred_json = model_json.is_fraud(&features).unwrap();
            let pred_onnx = model_onnx.is_fraud(&features).unwrap();

            assert_eq!(
                pred_json, expected_pred,
                "Sample {idx}: JSON prediction mismatch"
            );
            assert_eq!(
                pred_onnx, expected_pred,
                "Sample {idx}: ONNX prediction mismatch"
            );

            let bps = model_json.predict_risk_bps(&features).unwrap();
            assert!(bps <= 10_000);
            assert_eq!(bps, (expected_score * 10_000.0).round() as u32);
        }
    }

    #[test]
    fn test_missing_feature_rejected() {
        let model = FraudModel::load_from_json(MANIFEST_JSON).unwrap();
        let mut features = BTreeMap::new();
        features.insert("amount_minor".to_string(), 1000);

        let err = model.predict_score(&features).unwrap_err();
        match err {
            MlError::MissingFeature(_) => {}
            other => panic!("expected MissingFeature, got {other:?}"),
        }
    }
}
