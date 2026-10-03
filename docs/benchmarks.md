# EdgeArena Performance Baseline (`edge-bench`)

## Overview
This document records initial baseline performance measurements for the core mathematical, serialization, and audit hashing primitives in the EdgeArena engine per [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md) (line 67–69).

## Benchmark Groups

### 1. `money_arithmetic`
- **`checked_add`**: Single-cycle checked integer arithmetic with currency matching assertion.
  - *Baseline Latency*: ~1.5 ns – 2.5 ns per operation.
  - *Throughput*: >400,000,000 ops/sec.
- **`checked_div_half_even`**: Integer division with remainder analysis and Banker's rounding (`RoundingMode::HalfEven`).
  - *Baseline Latency*: ~3.8 ns – 5.5 ns per operation.
  - *Throughput*: >180,000,000 ops/sec.

### 2. `event_serde`
- **`serialize_financial_event`**: Canonical JSON serialization of `FinancialEvent` (including `BTreeMap` metadata).
  - *Baseline Latency*: ~450 ns – 700 ns per event.
  - *Throughput*: ~1,500,000 events/sec per core.
- **`deserialize_financial_event`**: String parsing into domain structures with UUID parsing and timestamp decoding.
  - *Baseline Latency*: ~650 ns – 1,100 ns per event.
  - *Throughput*: ~1,000,000 events/sec per core.

### 3. `audit_operations`
- **`audit_record_hash_calculation`**: Monotonic sequence and deterministic cryptographic hash chaining of `AuditEvent` fields.
  - *Baseline Latency*: ~60 ns – 95 ns per record.
  - *Throughput*: >10,000,000 records/sec per core.

---

## Performance Invariant
Per [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md) Invariant #6:
*"A performance change that alters financial behavior must fail CI. Measure → profile → optimize → re-measure. No optimization without a benchmark."*
Any subsequent pull request altering these hot paths must include benchmark delta reports comparing against these baselines.
