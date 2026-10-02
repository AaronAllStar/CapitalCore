# ADR-0001: Cargo Workspace Layout

## Context
The legacy EdgeArena system is structured as a Python monolith in a pnpm/Turborepo workspace (`apps/api`). The new architecture is a Rust-first financial intelligence engine per [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md). We require a crate architecture that strictly enforces the layered design (`API (thin) → Application → Domain → Intelligence Engine → Infrastructure`) with compile-time dependency enforcement, preventing domain logic from depending on HTTP, SQL, or I/O frameworks.

## Decision
Adopt a Cargo workspace containing 18 focused crates located under `crates/`:
- **Core / Domain (Zero I/O)**: `edge-core`, `edge-domain`, `edge-events`, `edge-transactions`, `edge-features`, `edge-rules`, `edge-detection`, `edge-ml`, `edge-decision`, `edge-risk`, `edge-audit`.
- **Infrastructure / Storage**: `edge-storage`, `edge-security`.
- **Presentation / Ingress**: `edge-api`, `edge-auth`.
- **Execution & Observability**: `edge-workers`, `edge-observability`, `edge-bench`.

Domain crates must NOT depend on Tokio, Axum, SQLx, or any external transport/database driver. Dependencies flow inward toward domain models.

## Alternatives Considered
1. **Single monolithic crate with internal modules**: Rejected because Rust modules cannot enforce compile-time dependency isolation or prevent accidental circular/I/O leaks into domain types.
2. **Coarse-grained multi-crate workspace (3–4 crates)**: Rejected because combining domain logic with transport abstractions weakens boundary enforcement and harms parallel compile times.
3. **Microservices from day one**: Rejected as premature complexity. A modular monolith within a single Cargo workspace provides strict boundaries with zero distributed networking overhead.

## Consequences
- **Positive**: Strict compile-time enforcement of layer boundaries; domain logic remains pure, deterministic, and easily testable without mocks; fine-grained caching and parallel compilation across crates.
- **Negative / Risks**: Managing multiple `Cargo.toml` manifests; inter-crate dependency coordination; requires disciplined design of public interfaces.

## Test Strategy
- CI gate checking `cargo deny` and package dependency graphs.
- Static architecture tests ensuring zero I/O or network crate references in `crates/edge-core`, `crates/edge-domain`, and related intelligence engine crates.

## Migration Impact
- Greenfield workspace created alongside frozen legacy Python reference code (`legacy/api/`).
- Zero breakage to existing legacy systems during initial crate initialization.
