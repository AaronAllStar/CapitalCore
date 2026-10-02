# EdgeArena Migration Status

## Current Status Overview
- **Active Phase**: Phase 1 — Audit & architecture
- **Active Task**: TASK-001 — Create `docs/STATUS.md`
- **Rule**: Only the phase marked `ACTIVE` may be worked on (per [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md)).

---

## Phases

| Phase | Name | Status | Started | Completed | Notes |
|---|---|---|---|---|---|
| 1 | Audit & architecture | **ACTIVE** | 2026-10-02 | — | Full codebase audit complete; establishing baseline documentation and ADRs. |
| 2 | Domain model | NOT_STARTED | — | — | Workspace init, monetary types, core domain types, audit/event bus abstractions. |
| 3 | Golden datasets | NOT_STARTED | — | — | Golden test formats, dataset runners, CI pipeline with quality gates. |
| 4 | Core | NOT_STARTED | — | — | SQLx storage, Argon2id/Ed25519 auth, observability, rate limiting & error models. |
| 5 | Event engine | NOT_STARTED | — | — | Validation, normalization, in-memory event processing pipeline. |
| 6 | Rules | NOT_STARTED | — | — | Rule engine, AST/interpreter or compiled DSL, deterministic rule evaluation. |
| 7 | Features | NOT_STARTED | — | — | Real-time feature calculation, aggregations, sliding windows. |
| 8 | Decision | NOT_STARTED | — | — | Decision engine (ALLOW, REVIEW, BLOCK, ESCALATE), immutable audit generation. |
| 9 | API | NOT_STARTED | — | — | Thin Axum API layer, JWT middleware, transaction submission, OpenAPI spec. |
| 10 | Workers/events | NOT_STARTED | — | — | Async workers, queue evaluation (Tokio tasks / NATS JetStream). |
| 11 | ML training | NOT_STARTED | — | — | Python training pipeline, feature schema alignment, ONNX export. |
| 12 | Rust inference | NOT_STARTED | — | — | `edge-ml` ONNX Runtime integration in Rust execution path. |
| 13 | Security hardening | NOT_STARTED | — | — | Penetration test remediation, fuzzing, secret scrubbing, sandbox isolation. |
| 14 | Perf | NOT_STARTED | — | — | Criterion benchmark deltas, profiling, latency optimization (p99/p99.9). |
| 15 | Load/stress | NOT_STARTED | — | — | End-to-end stress testing, concurrency thresholds, degradation behavior. |
| 16 | Production readiness | NOT_STARTED | — | — | Deployment configs, operational runbooks, final verification. |

---

## Phase 1 Tasks

| Task ID | Goal | Status | Assigned |
|---|---|---|---|
| **TASK-001** | Create `docs/STATUS.md` with phase tracking and mark Phase 1 as ACTIVE | **IN_PROGRESS** | Implementer |
| **TASK-002** | Freeze legacy Python code as a read-only reference in `legacy/api/` | NOT_STARTED | Implementer |
| **TASK-003** | Set up ADR directory and create first 4 ADRs (0001–0004) from audit | NOT_STARTED | Implementer |

---

## Change Log
- **2026-10-02**: Initialized `docs/STATUS.md` under TASK-001; Phase 1 marked ACTIVE.
