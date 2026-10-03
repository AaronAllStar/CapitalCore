"""
Synthetic financial transaction dataset generator for fraud model training.
Emits feature vectors matching ml/schema/feature_schema.json with labeled fraud outcomes.
Fully deterministic using seeded random generator.
"""

import os
import json
import random
from typing import List, Dict, Any
from feature_extractor import PythonFeatureEngine


def generate_dataset(num_transactions: int = 1250, seed: int = 42) -> List[Dict[str, Any]]:
    random.seed(seed)

    engine = PythonFeatureEngine()
    users = [f"user_{i:03d}" for i in range(1, 31)]
    merchants = [f"merchant_{i:03d}" for i in range(1, 21)]

    # Start epoch timestamp
    base_timestamp = 1_700_000_000
    dataset = []

    # Track time per user
    user_times = {u: base_timestamp + random.randint(0, 3600) for u in users}

    for idx in range(num_transactions):
        user = random.choice(users)
        current_time = user_times[user]

        # 10% probability of fraud pattern
        is_fraud_scenario = random.random() < 0.10

        if is_fraud_scenario:
            fraud_type = random.choice(["velocity_spike", "large_amount", "merchant_hopping"])
            if fraud_type == "velocity_spike":
                # Rapid successive transactions
                advance = random.randint(5, 30)
                amount = random.randint(15000, 80000) # $150 - $800
                merchant = random.choice(merchants[:3])
            elif fraud_type == "large_amount":
                # Sudden massive transaction
                advance = random.randint(300, 3600)
                amount = random.randint(600000, 2500000) # $6,000 - $25,000
                merchant = random.choice(merchants)
            else: # merchant_hopping
                advance = random.randint(10, 60)
                amount = random.randint(5000, 35000)
                merchant = f"foreign_merchant_{random.randint(100, 999)}"

            label = 1
        else:
            # Legitimate shopping pattern
            advance = random.randint(1800, 28800) # 30 mins to 8 hours
            amount = random.randint(500, 25000) # $5 - $250
            merchant = random.choice(merchants)
            label = 0

        user_times[user] = current_time + advance
        timestamp = user_times[user]

        # Extract features using sliding window engine
        features = engine.compute_and_update(
            user_id=user,
            amount_minor=amount,
            timestamp_secs=timestamp,
            metadata={"merchant_id": merchant},
        )

        record = dict(features)
        record["is_fraud"] = label
        record["user_id"] = user
        record["timestamp_secs"] = timestamp
        record["merchant_id"] = merchant

        dataset.append(record)

    return dataset


def main():
    output_dir = os.path.join(os.path.dirname(__file__), "data")
    os.makedirs(output_dir, exist_ok=True)

    all_data = generate_dataset(num_transactions=1250, seed=42)

    # 80/20 train/test split
    train_data = all_data[:1000]
    test_data = all_data[1000:]

    train_path = os.path.join(output_dir, "train_features.jsonl")
    with open(train_path, "w", encoding="utf-8") as f:
        for r in train_data:
            f.write(json.dumps(r) + "\n")

    test_path = os.path.join(output_dir, "test_features.jsonl")
    with open(test_path, "w", encoding="utf-8") as f:
        for r in test_data:
            f.write(json.dumps(r) + "\n")

    fraud_count_train = sum(r["is_fraud"] for r in train_data)
    fraud_count_test = sum(r["is_fraud"] for r in test_data)

    print(f"Generated {len(train_data)} training samples ({fraud_count_train} fraud) -> {train_path}")
    print(f"Generated {len(test_data)} test samples ({fraud_count_test} fraud) -> {test_path}")


if __name__ == "__main__":
    main()
