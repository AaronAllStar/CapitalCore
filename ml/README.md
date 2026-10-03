# EdgeArena Machine Learning Pipeline

## Overview
Phase 11 establishes the offline training data pipeline and feature extraction framework for fraud detection. The feature definitions and sliding window temporal mechanics match the Rust `edge-features` crate with 100% mathematical parity.

## Features Catalog
All monetary amounts and temporal counts use pure integer arithmetic:

| Feature Name | Type | Window | Description |
|---|---|---|---|
| `amount_minor` | `int64` | Instant | Transaction amount in minor currency units (cents) |
| `user_txn_count_5m` | `int64` | 5 minutes | Trailing transaction count |
| `user_txn_sum_5m` | `int64` | 5 minutes | Trailing aggregate amount in minor units |
| `user_txn_count_1h` | `int64` | 1 hour | Trailing transaction count |
| `user_txn_sum_1h` | `int64` | 1 hour | Trailing aggregate amount in minor units |
| `user_txn_max_1h` | `int64` | 1 hour | Maximum individual transaction amount |
| `user_txn_count_24h` | `int64` | 24 hours | Trailing transaction count |
| `user_txn_sum_24h` | `int64` | 24 hours | Trailing aggregate amount in minor units |
| `user_distinct_merchants_24h` | `int64` | 24 hours | Distinct merchant identifiers observed |

Target variable: `is_fraud` (binary integer: `0` = legitimate, `1` = fraud).

## Parity Verification
The Python implementation in `feature_extractor.py` is tested against the Rust `edge-features` test suite in `tests/test_feature_parity.py`:
```bash
python ml/tests/test_feature_parity.py
```

## Dataset Generation
Generate training (`1,000` samples) and test (`250` samples) datasets:
```bash
python ml/dataset_generator.py
```
Output files:
- `ml/data/train_features.jsonl`
- `ml/data/test_features.jsonl`

## Model Training & ONNX Export
Train the fraud classification decision tree:
```bash
python ml/train.py
```
Outputs:
- `ml/models/model_manifest.json`: Architecture, split nodes, decision thresholds.
- `ml/models/fraud_detector.onnx`: ONNX format model artifact.

## Model Evaluation
Evaluate on the holdout test set:
```bash
python ml/evaluate.py
```

### Metrics Benchmark
- **Accuracy**: `99.60%`
- **Precision**: `96.55%`
- **Recall**: `100.00%`
- **F1 Score**: `98.25%`
- **Sample Inferences**: `ml/models/sample_inferences.json` (Used for Rust Phase 12 validation).

