"""
Pure Python sliding window feature extractor.
Guarantees 1:1 mathematical parity with crates/edge-features/src/pipeline.rs.
Zero floating-point conversions; integer arithmetic only.
"""

from typing import Dict, List, Any, Optional
from collections import defaultdict


class WindowEvent:
    def __init__(self, timestamp_secs: int, amount_minor: int, metadata: Dict[str, str]):
        self.timestamp_secs = timestamp_secs
        self.amount_minor = amount_minor
        self.metadata = metadata


class UserEventWindow:
    def __init__(self):
        self.events: List[WindowEvent] = []

    def count_since(self, since_secs: int) -> int:
        return sum(1 for e in self.events if e.timestamp_secs >= since_secs)

    def sum_since(self, since_secs: int) -> int:
        return sum(e.amount_minor for e in self.events if e.timestamp_secs >= since_secs)

    def max_since(self, since_secs: int) -> int:
        amounts = [e.amount_minor for e in self.events if e.timestamp_secs >= since_secs]
        return max(amounts) if amounts else 0

    def distinct_metadata_since(self, since_secs: int, key: str) -> int:
        seen = set()
        for e in self.events:
            if e.timestamp_secs >= since_secs and key in e.metadata:
                seen.add(e.metadata[key])
        return len(seen)

    def record(self, timestamp_secs: int, amount_minor: int, metadata: Dict[str, str]) -> None:
        self.events.append(WindowEvent(timestamp_secs, amount_minor, metadata))
        # Keep sorted by timestamp
        self.events.sort(key=lambda e: e.timestamp_secs)

    def prune_before(self, cutoff_secs: int) -> None:
        self.events = [e for e in self.events if e.timestamp_secs >= cutoff_secs]


class PythonFeatureEngine:
    """Mirrors edge_features::FeatureEngine."""

    def __init__(self):
        self.windows: Dict[str, UserEventWindow] = defaultdict(UserEventWindow)

    def compute_and_update(
        self,
        user_id: str,
        amount_minor: int,
        timestamp_secs: int,
        metadata: Optional[Dict[str, str]] = None,
    ) -> Dict[str, int]:
        if metadata is None:
            metadata = {}

        window = self.windows[user_id]

        # 1. Compute features prior to inserting the new event
        features = {
            "amount_minor": amount_minor,
            "user_txn_count_5m": window.count_since(timestamp_secs - 300),
            "user_txn_sum_5m": window.sum_since(timestamp_secs - 300),
            "user_txn_count_1h": window.count_since(timestamp_secs - 3600),
            "user_txn_sum_1h": window.sum_since(timestamp_secs - 3600),
            "user_txn_max_1h": window.max_since(timestamp_secs - 3600),
            "user_txn_count_24h": window.count_since(timestamp_secs - 86400),
            "user_txn_sum_24h": window.sum_since(timestamp_secs - 86400),
            "user_distinct_merchants_24h": window.distinct_metadata_since(
                timestamp_secs - 86400, "merchant_id"
            ),
        }

        # 2. Record new event into window
        window.record(timestamp_secs, amount_minor, metadata)

        # 3. Prune events older than 24 hours
        window.prune_before(timestamp_secs - 86400)

        return features
