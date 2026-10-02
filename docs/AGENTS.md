# EdgeArena — Financial Intelligence Platform

## Mission
Rewrite EdgeArena (currently a Python trading/backtesting platform) as a Rust-first
financial intelligence engine for banks, fintechs and payment processors:
real-time transaction monitoring, fraud/anomaly detection, behavioral analytics,
explainable decisions, full auditability.

Out of scope: credit scoring / credit risk. Trading features are NOT preserved by default.

## Core pipeline
FinancialEvent → Validate → Normalize → Features → Rules → Behavioral → ML
→ Decision (ALLOW | REVIEW | BLOCK | ESCALATE) → AuditEvent

Deterministic rules/policy are ALWAYS authoritative over ML output.

## Priority order (breaks every tie)
Correctness > Security > Determinism > Performance > Scalability > Maintainability

## Language boundary
- Rust: API, auth, domain, events, rules, features, decision, workers, storage,
  audit, observability, benchmarks, CLI, ML inference.
- Python: ONLY ml/training (research, training, notebooks). Never in the production path.
- Flow: Python training → model artifact (ONNX or equivalent) → Rust inference.
- Forbidden: Rust → Python service → Rust at runtime.
- FastAPI/Celery/Python workers are replaced, not wrapped.

## Architecture
Layers: API (thin) → Application → Domain → Intelligence Engine → Infrastructure.
Domain crates must NOT depend on HTTP, SQL or any I/O framework.
Cargo workspace; initial crate proposal (may be changed via ADR):
edge-domain, edge-core, edge-events, edge-transactions, edge-features, edge-rules,
edge-detection, edge-ml, edge-decision, edge-risk, edge-audit, edge-storage,
edge-api, edge-auth, edge-security, edge-workers, edge-observability, edge-bench.
Default stack to evaluate: Tokio, Axum, Tower, Serde, SQLx, tracing.
Infra (Postgres/Redis/NATS/Kafka/object storage) only when a measured requirement justifies it.

## Hard invariants (CI must enforce)
1. No f32/f64 for money. Use integer minor units or a decimal type, explicit rounding
   mode, per-currency precision. Every numeric decision is documented in an ADR.
2. Same input → same output (deterministic). No wall-clock, RNG or HashMap iteration
   order leaking into decisions; time is injected.
3. Strong types over strings: states, currencies, channels, event types, decisions,
   alert/investigation/model states are enums or newtypes.
4. Every decision is reconstructable: inputs, feature versions, rule versions,
   model version, outcome, reasons → immutable AuditEvent.
5. All external input is untrusted (payloads, datasets, rules, model artifacts,
   uploads, config). Validate at the boundary. Never execute untrusted code in the
   main process; custom logic requires a sandbox (e.g. Wasm).
6. A performance change that alters financial behavior must fail CI.
7. No `unsafe` without an ADR and a justification comment.

## Migration rules
- Audit first, rewrite second. Legacy Python is the reference implementation:
  do not delete any legacy component until golden/differential tests pass.
- Order: golden datasets/results → perf baseline → security baseline →
  domain model → Rust workspace → incremental migration.
- Every legacy component gets one label: KEEP | REFACTOR | GENERALIZE | REPLACE | REMOVE.
- Do not preserve bad design to ease migration.

## Quality gates (per PR)
cargo fmt --check · cargo clippy -- -D warnings · cargo test ·
cargo deny / cargo audit · golden + differential tests · property tests for
financial logic · benchmark delta report for hot paths.
Measure → profile → optimize → re-measure. No optimization without a benchmark.

## Metrics tracked in EdgeBench
tx/s, events/s, rules/s, inferences/s, p50/p95/p99/p99.9 latency,
memory, CPU, concurrent accounts/streams.

## Phases
1 Audit & architecture · 2 Domain model · 3 Golden datasets · 4 Core ·
5 Event engine · 6 Rules · 7 Features · 8 Decision · 9 API · 10 Workers/events ·
11 ML training · 12 Rust inference · 13 Security hardening · 14 Perf ·
15 Load/stress · 16 Production readiness.
Only the phase marked ACTIVE in `docs/STATUS.md` may be worked on.