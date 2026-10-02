# Golden Test Datasets

## Overview
Golden test datasets serve as the deterministic reference benchmarks for the EdgeArena financial intelligence engine.
Per [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md) Invariants #2 and #4:
- All decisions are deterministic: `Same input → same output`.
- Inputs and expected outputs are version-controlled in JSON Lines (`.jsonl`) format.
- Datasets test the full boundary from raw ingested `FinancialEvent` to policy `Decision`.

## Directory Structure
- `golden/datasets/`: Input events in JSON Lines (`.jsonl`) format. One `FinancialEvent` per line.
- `golden/expected/`: Expected evaluation outcomes in JSON Lines (`.jsonl`) format, keyed by `event_id`.

## Schema Specifications

### Input Event Schema (`FinancialEvent`)
Each line in `golden/datasets/*.jsonl` must deserialize directly to `edge_domain::FinancialEvent`:
```json
{
  "event_id": "UUID",
  "transaction_id": "UUID",
  "user_id": "UUID",
  "event_type": "Payment | Transfer | Withdrawal | Deposit | Refund | Chargeback | Authorization",
  "channel": "Web | Mobile | Api | Pos | Atm | Batch",
  "money": {
    "amount": 1000,
    "currency": "USD"
  },
  "created_at": "RFC3339 timestamp (e.g. 2026-10-02T12:00:00Z)",
  "metadata": {
    "key": "value"
  }
}
```

### Expected Output Schema (`ExpectedDecision`)
Each line in `golden/expected/*.jsonl` corresponds by `event_id` to an input event:
```json
{
  "event_id": "UUID",
  "decision": "Allow | Review | Block | Escalate",
  "risk_level": "Low | Medium | High | Critical",
  "reasons": ["string explanation"],
  "triggered_rules": ["RuleId UUID"],
  "model_id": null
}
```

## Privacy & Security Note
No production PII or real credentials may be placed in golden files. All identifiers are synthetic UUIDs.
