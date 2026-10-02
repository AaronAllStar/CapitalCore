# Legacy Python Reference Code (`legacy/api`)

## Purpose
This directory contains a frozen snapshot of the legacy Python FastAPI/Celery backend (`apps/api`) preserved as a reference implementation during the EdgeArena Rust migration.

## Migration Policy & Invariants
Per [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md) and [WORKFLOW.md](file:///c:/Users/damed/Downloads/edge-arena/docs/WORKFLOW.md):
- **Read-Only / Frozen**: Do NOT modify files in this directory.
- **Reference Only**: Used for golden dataset generation, logic verification, and differential testing against the Rust engine.
- **No Deletion**: Do not delete any legacy component until golden/differential tests pass in Phase 3.
- **Security**: Environment configurations (`.env`) and secrets are explicitly excluded from this directory.
