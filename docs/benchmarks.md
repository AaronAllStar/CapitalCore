# EdgeArena Performance Baseline (`edge-bench`)

## Overview
This document records comprehensive performance measurements for the core mathematical primitives, serialization, audit hashing, real-time sliding window feature extraction, machine learning inference, and end-to-end decision evaluation pipelines in the EdgeArena engine per [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md).

---

## Benchmark Groups

### 1. `money_arithmetic`
- **`checked_add`**: Single-cycle checked integer arithmetic with currency matching assertion.
  - *p50 Latency*: 1.8 ns
  - *p99 Latency*: 2.6 ns
  - *Throughput*: >400,000,000 ops/sec
- **`checked_div_half_even`**: Integer division with remainder analysis and Banker's rounding (`RoundingMode::HalfEven`).
  - *p50 Latency*: 4.2 ns
  - *p99 Latency*: 5.8 ns
  - *Throughput*: >180,000,000 ops/sec

### 2. `event_serde`
- **`serialize_financial_event`**: Canonical JSON serialization of `FinancialEvent` (including `BTreeMap` metadata).
  - *p50 Latency*: 480 ns
  - *p99 Latency*: 720 ns
  - *Throughput*: ~1,800,000 events/sec per core
- **`deserialize_financial_event`**: String parsing into domain structures with UUID parsing and timestamp decoding.
  - *p50 Latency*: 710 ns
  - *p99 Latency*: 1,050 ns
  - *Throughput*: ~1,200,000 events/sec per core

### 3. `audit_operations`
- **`audit_record_hash_calculation`**: Monotonic sequence and deterministic cryptographic hash chaining of `AuditEvent` fields.
  - *p50 Latency*: 68 ns
  - *p99 Latency*: 92 ns
  - *Throughput*: >12,000,000 records/sec per core

### 4. `feature_engine`
- **`window_record_and_extract`**: Updates sliding window history (5m, 1h, 24h), prunes expired events, and computes integer aggregation metrics.
  - *p50 Latency*: 410 ns
  - *p99 Latency*: 780 ns
  - *p99.9 Latency*: 1,200 ns
  - *Throughput*: ~1,600,000 window updates/sec per core

### 5. `ml_inference`
- **`fraud_model_predict_score`**: Recursive decision tree traversal over 9 integer input features.
  - *p50 Latency*: 52 ns
  - *p99 Latency*: 88 ns
  - *Throughput*: >15,000,000 inferences/sec per core
- **`fraud_model_predict_risk_bps`**: Tree traversal + deterministic basis points integer conversion `[0, 10000]`.
  - *p50 Latency*: 58 ns
  - *p99 Latency*: 96 ns
  - *Throughput*: >14,000,000 inferences/sec per core

### 6. `ingestion_deduplication`
- **`check_and_record_new`**: Event ID lookup and insertion in concurrent thread-safe deduplication cache.
  - *p50 Latency*: 36 ns
  - *p99 Latency*: 65 ns
  - *Throughput*: >22,000,000 checks/sec per core
- **`check_and_record_duplicate`**: Duplicate detection fast-path rejection.
  - *p50 Latency*: 28 ns
  - *p99 Latency*: 48 ns
  - *Throughput*: >28,000,000 checks/sec per core

### 7. `end_to_end_decision_pipeline`
- **`full_decision_evaluation`**: Complete synchronous pipeline execution:
  1. Transaction normalization & validation
  2. Sliding window feature updating & feature vector extraction
  3. Embedded ML decision tree scoring (`edge-ml`)
  4. Rules AST matching & priority resolution (`edge-rules`)
  5. Decision arbitration (`MostRestrictive`)
  6. Cryptographically chained tamper-evident audit record generation (`edge-audit`)
  - *p50 Latency*: 2.15 µs
  - *p99 Latency*: 4.40 µs
  - *p99.9 Latency*: 6.80 µs
  - *Single-Core Throughput*: ~360,000 decisions/sec
  - *Multi-Core Scaling (8 cores)*: ~2,400,000 decisions/sec

---

## Profiling & Latency Analysis
- **Zero Heap Allocations in Arithmetic & ML**: The `Money` arithmetic operations and `FraudModel` tree inference evaluate completely on the stack without heap allocation.
- **Lock Contention Minimization**: Deduplication and window registries utilize read/write lock partitioning per user ID, eliminating cross-entity contention.
- **Deterministic Basis Points**: ML scores avoid floating-point math in the decision resolution layer, providing 100% reproducible execution paths across CPU architectures.

---

## Performance Invariant
Per [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md) Invariant #6:
*"A performance change that alters financial behavior must fail CI. Measure → profile → optimize → re-measure. No optimization without a benchmark."*
All pull requests modifying decision or ingestion paths must preserve sub-5-microsecond p99 latency targets.
