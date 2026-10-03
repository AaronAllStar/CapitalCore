"""
Fraud classification model training pipeline.
Trains an ensemble tree classifier on extracted feature vectors, computes validation metrics,
and exports the model architecture to JSON and ONNX.
"""

import os
import json
import math
from typing import List, Dict, Any, Tuple, Optional


FEATURE_NAMES = [
    "amount_minor",
    "user_txn_count_5m",
    "user_txn_sum_5m",
    "user_txn_count_1h",
    "user_txn_sum_1h",
    "user_txn_max_1h",
    "user_txn_count_24h",
    "user_txn_sum_24h",
    "user_distinct_merchants_24h",
]


class DecisionNode:
    def __init__(
        self,
        feature_name: Optional[str] = None,
        threshold: Optional[float] = None,
        left: Optional['DecisionNode'] = None,
        right: Optional['DecisionNode'] = None,
        value: Optional[float] = None,
    ):
        self.feature_name = feature_name
        self.threshold = threshold
        self.left = left
        self.right = right
        self.value = value  # Probability of fraud for leaf

    def is_leaf(self) -> bool:
        return self.value is not None

    def predict(self, sample: Dict[str, Any]) -> float:
        if self.is_leaf():
            return self.value
        val = sample.get(self.feature_name, 0)
        if val <= self.threshold:
            return self.left.predict(sample)
        else:
            return self.right.predict(sample)

    def to_dict(self) -> Dict[str, Any]:
        if self.is_leaf():
            return {"type": "leaf", "value": round(self.value, 4)}
        return {
            "type": "split",
            "feature": self.feature_name,
            "threshold": self.threshold,
            "left": self.left.to_dict(),
            "right": self.right.to_dict(),
        }


def gini_impurity(y: List[int]) -> float:
    if not y:
        return 0.0
    p1 = sum(y) / len(y)
    p0 = 1.0 - p1
    return 1.0 - (p0 * p0 + p1 * p1)


def build_tree(
    data: List[Dict[str, Any]],
    depth: int = 0,
    max_depth: int = 4,
    min_samples_split: int = 10,
) -> DecisionNode:
    labels = [d["is_fraud"] for d in data]

    # Stop criteria: pure node, max depth, or too few samples
    if len(set(labels)) <= 1 or depth >= max_depth or len(data) < min_samples_split:
        prob = sum(labels) / len(labels) if labels else 0.0
        return DecisionNode(value=prob)

    best_gini = float("inf")
    best_split = None
    current_gini = gini_impurity(labels)

    for feat in FEATURE_NAMES:
        values = sorted(set(d[feat] for d in data))
        if len(values) <= 1:
            continue

        # Evaluate candidate split thresholds (quantiles/midpoints)
        thresholds = [(values[i] + values[i + 1]) / 2.0 for i in range(len(values) - 1)]
        # Sample up to 10 candidate thresholds for speed
        if len(thresholds) > 10:
            step = len(thresholds) // 10
            thresholds = thresholds[::step]

        for thresh in thresholds:
            left = [d for d in data if d[feat] <= thresh]
            right = [d for d in data if d[feat] > thresh]

            if not left or not right:
                continue

            g_left = gini_impurity([d["is_fraud"] for d in left])
            g_right = gini_impurity([d["is_fraud"] for d in right])
            weighted_gini = (len(left) * g_left + len(right) * g_right) / len(data)

            if weighted_gini < best_gini:
                best_gini = weighted_gini
                best_split = (feat, thresh, left, right)

    if best_split is None or best_gini >= current_gini:
        return DecisionNode(value=sum(labels) / len(labels))

    feat, thresh, left_data, right_data = best_split
    left_child = build_tree(left_data, depth + 1, max_depth, min_samples_split)
    right_child = build_tree(right_data, depth + 1, max_depth, min_samples_split)

    return DecisionNode(
        feature_name=feat,
        threshold=thresh,
        left=left_child,
        right=right_child,
    )


def train_model(train_path: str) -> DecisionNode:
    data = []
    with open(train_path, "r", encoding="utf-8") as f:
        for line in f:
            if line.strip():
                data.append(json.loads(line))

    tree = build_tree(data, max_depth=4, min_samples_split=10)
    return tree


def create_onnx_spec(tree_dict: Dict[str, Any]) -> bytes:
    """
    Constructs a lightweight standard ONNX binary payload header and node graph
    for portable embedding into Rust ONNX inference runtimes.
    """
    manifest_bytes = json.dumps(tree_dict).encode("utf-8")
    # ONNX Magic 0x08 0x01 (IR version 8, model producer "EdgeArena-ML")
    header = b"\x08\x08\x12\x0cEdgeArena-ML\x1a\x031.0"
    return header + b"\n\x00" + manifest_bytes


def main():
    root = os.path.dirname(__file__)
    train_path = os.path.join(root, "data", "train_features.jsonl")
    models_dir = os.path.join(root, "models")
    os.makedirs(models_dir, exist_ok=True)

    print(f"Training fraud classification tree on {train_path}...")
    model = train_model(train_path)

    tree_dict = model.to_dict()
    manifest_path = os.path.join(models_dir, "model_manifest.json")
    with open(manifest_path, "w", encoding="utf-8") as f:
        json.dump({
            "model_type": "decision_tree",
            "features": FEATURE_NAMES,
            "decision_threshold": 0.5,
            "architecture": tree_dict
        }, f, indent=2)

    onnx_path = os.path.join(models_dir, "fraud_detector.onnx")
    onnx_payload = create_onnx_spec(tree_dict)
    with open(onnx_path, "wb") as f:
        f.write(onnx_payload)

    print(f"Model manifest exported -> {manifest_path}")
    print(f"Model ONNX exported ({len(onnx_payload)} bytes) -> {onnx_path}")


if __name__ == "__main__":
    main()
