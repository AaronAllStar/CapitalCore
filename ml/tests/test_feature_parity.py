"""
Parity test suite ensuring Python feature extraction identically matches Rust edge-features engine.
"""

import sys
import os
import json

# Add parent directory to sys.path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from feature_extractor import PythonFeatureEngine
from dataset_generator import generate_dataset


def test_rust_equivalent_trailing_velocity_sequence():
    """
    Directly mirrors crates/edge-features/src/lib.rs:
    test_feature_engine_trailing_velocity_sequence
    """
    engine = PythonFeatureEngine()
    user_id = "test_user_001"
    base_time = 1_700_000_000

    # Transaction 1: amount = 1000, t = base_time, merch_1
    f1 = engine.compute_and_update(user_id, 1000, base_time, {"merchant_id": "merch_1"})
    assert f1["user_txn_count_1h"] == 0, f"Expected 0, got {f1['user_txn_count_1h']}"
    assert f1["user_txn_sum_1h"] == 0, f"Expected 0, got {f1['user_txn_sum_1h']}"
    assert f1["amount_minor"] == 1000

    # Transaction 2: amount = 2500, t = base_time + 120s (2 mins), merch_2
    f2 = engine.compute_and_update(user_id, 2500, base_time + 120, {"merchant_id": "merch_2"})
    assert f2["user_txn_count_1h"] == 1, f"Expected 1, got {f2['user_txn_count_1h']}"
    assert f2["user_txn_sum_1h"] == 1000, f"Expected 1000, got {f2['user_txn_sum_1h']}"
    assert f2["user_txn_count_5m"] == 1, f"Expected 1, got {f2['user_txn_count_5m']}"

    # Transaction 3: amount = 5000, t = base_time + 240s (4 mins), merch_1
    f3 = engine.compute_and_update(user_id, 5000, base_time + 240, {"merchant_id": "merch_1"})
    assert f3["user_txn_count_1h"] == 2, f"Expected 2, got {f3['user_txn_count_1h']}"
    assert f3["user_txn_sum_1h"] == 3500, f"Expected 3500, got {f3['user_txn_sum_1h']}"
    assert f3["user_txn_max_1h"] == 2500, f"Expected 2500, got {f3['user_txn_max_1h']}"
    assert f3["user_distinct_merchants_24h"] == 2, f"Expected 2, got {f3['user_distinct_merchants_24h']}"

    print("PASS: test_rust_equivalent_trailing_velocity_sequence")


def test_schema_conformance():
    schema_path = os.path.join(os.path.dirname(__file__), "..", "schema", "feature_schema.json")
    with open(schema_path, "r", encoding="utf-8") as f:
        schema = json.load(f)

    required_props = set(schema["required"])
    data = generate_dataset(num_transactions=10, seed=123)

    for item in data:
        for prop in required_props:
            assert prop in item, f"Missing required property {prop}"
            val = item[prop]
            assert isinstance(val, int), f"Property {prop} must be int, got {type(val)}"
            assert val >= 0, f"Property {prop} must be non-negative, got {val}"

    print("PASS: test_schema_conformance")


if __name__ == "__main__":
    test_rust_equivalent_trailing_velocity_sequence()
    test_schema_conformance()
    print("ALL TESTS PASSED SUCCESSFULLY.")
