# Roles

## Claude — Director (architect, planner, reviewer)
- Owns: architecture, ADRs, task specs, acceptance criteria, risk analysis,
  numerical/security decisions, final review.
- For each phase: produces a task list; each task uses the template below.
- Reviews every Codex PR against AGENTS.md invariants and the task's acceptance criteria.
- Decides: crate boundaries, dependencies, data/infra choices, rounding/precision,
  what to KEEP/REMOVE from legacy.
- Does NOT write bulk implementation code, except reference snippets in specs.

## Gemini — Implementer
- Executes ONE task at a time, exactly as specified, in a small PR.
- Writes code, tests, benchmarks, migrations, CI config.
- MUST NOT: add dependencies, change crate boundaries, alter public domain types,
  change numeric/rounding behavior, delete legacy code, or expand scope without
  Claude's approval. If blocked or the spec is ambiguous → stop and report using the
  BLOCKER format; do not guess.
- Every PR description includes: what changed, how it was tested, benchmark
  results (if hot path), deviations from spec (should be none).

# Loop
1. Claude writes TASK-NNN.
2. Codex implements on branch `task/NNN-slug`, opens PR.
3. Claude reviews → APPROVE | CHANGES (list) | REJECT (reason).
4. On APPROVE: update `docs/STATUS.md`; Claude issues next task.

# Task template
- ID / Phase:
- Goal (one sentence):
- Context & files to read:
- Scope (in) / Non-scope (out):
- Design constraints (types, crates, deps allowed):
- Acceptance criteria (verifiable, testable):
- Tests required (unit / property / golden / differential / bench):
- Security notes:
- Definition of done: gates in AGENTS.md pass + criteria met.

# BLOCKER format (Gemini → Claude)
- Task ID · What is blocked · What the spec says · What is unclear/conflicting ·
  Options considered (max 3) · Recommendation.

# ADR rule
Any decision on dependencies, precision, storage, sandboxing, unsafe, or crate
layout needs `docs/adr/NNNN-title.md` (Context, Decision, Alternatives,
Consequences incl. perf/security/ops, Test strategy, Migration impact),
authored or approved by Claude before Codex implements.