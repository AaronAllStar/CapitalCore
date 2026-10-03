"""
Model validation and evaluation script.
Computes accuracy, precision, recall, and F1 on holdout test set,
and writes reference inferences for Rust verification.
"""

import os
import json
from typing import Dict, Any


def evaluate_node(node: Dict[str, Any], sample: Dict[str, Any]) -> float:
    if node["type"] == "leaf":
        return node["value"]
    val = sample.get(node["feature"], 0)
    if val <= node["threshold"]:
        return evaluate_node(node["left"], sample)
    else:
        return evaluate_node(node["right"], sample)


def main():
    root = os.path.dirname(__file__)
    test_path = os.path.join(root, "data", "test_features.jsonl")
    manifest_path = os.path.join(root, "models", "model_manifest.json")

    with open(manifest_path, "r", encoding="utf-8") as f:
        manifest = json.load(f)

    tree = manifest["architecture"]
    threshold = manifest["decision_threshold"]

    samples = []
    with open(test_path, "r", encoding="utf-8") as f:
        for line in f:
            if line.strip():
                samples.append(json.loads(line))

    tp = fp = tn = fn = 0
    predictions = []

    for s in samples:
        score = evaluate_node(tree, s)
        pred = 1 if score >= threshold else 0
        actual = s["is_fraud"]

        if pred == 1 and actual == 1:
            tp += 1
        elif pred == 1 and actual == 0:
            fp += 1
        elif pred == 0 and actual == 0:
            tn += 1
        elif pred == 0 and actual == 1:
            fn += 1

        predictions.append({
            "features": {k: s[k] for k in manifest["features"]},
            "fraud_score": round(score, 4),
            "prediction": pred,
            "actual": actual
        })

    total = len(samples)
    accuracy = (tp + tn) / total if total > 0 else 0.0
    precision = tp / (tp + fp) if (tp + fp) > 0 else 0.0
    recall = tp / (tp + fn) if (tp + fn) > 0 else 0.0
    f1 = 2 * (precision * recall) / (precision + recall) if (precision + recall) > 0 else 0.0

    print("=================== EVALUATION RESULTS ===================")
    print(f"Total Test Samples: {total}")
    print(f"True Positives:     {tp}")
    print(f"False Positives:    {fp}")
    print(f"True Negatives:     {tn}")
    print(f"False Negatives:    {fn}")
    print(f"Accuracy:           {accuracy:.4f}")
    print(f"Precision:          {precision:.4f}")
    print(f"Recall:             {recall:.4f}")
    print(f"F1 Score:           {f1:.4f}")
    print("==========================================================")

    # Save reference inferences for Rust phase 12
    samples_path = os.path.join(root, "models", "sample_inferences.json")
    with open(samples_path, "w", encoding="utf-8") as f:
        json.dump({
            "metrics": {
                "accuracy": round(accuracy, 4),
                "precision": round(precision, 4),
                "recall": round(recall, 4),
                "f1": round(f1, 4)
            },
            "samples": predictions[:10]  # 10 golden samples for Rust integration
        }, f, indent=2)

    print(f"Golden inference samples saved -> {samples_path}")


if __name__ == "__main__":
    main()
