---
change: mamba-756-patrol
group: default
date: 2026-04-11
status: skipped
source: structured-issue
---

# Post-Clarifications

## Scope Summary

### Problem
-> See requirements.md

### Success Criteria
-> See requirements.md

### Boundary
- **In scope**: - **In scope**:
  - `crates/mamba/src/runtime/generator.rs` — semantic layer
  - `crates/mamba/src/runtime/async_generator.rs` — if separate
  - `crates/mamba/tests/conformance/generators/` — ported CPython
    tests (`test_generators.py` subset)
- **Out of scope**:
  - **Generator perf** — #1187 owns the 5x slower problem; this issue
    must stay green after that rewrite lands, nothing more
  - `asyncio` task wrapping — the async generator protocol alone is in
    scope, not the surrounding event loop semantics
  - `contextlib.contextmanager` conformance — separate stdlib issue

