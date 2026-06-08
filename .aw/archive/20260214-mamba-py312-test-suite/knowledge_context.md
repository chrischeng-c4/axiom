---
change_id: mamba-py312-test-suite
type: knowledge_context
created_at: 2026-02-13T10:32:16.009711+00:00
updated_at: 2026-02-13T10:32:16.009711+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - orbit
  - spec-to-code
---

# Knowledge Context

## Relevant Documents

- **knowledge:orbit/bridge-internals.md**
  - summary: Details GIL release strategy, exception translation between Rust/Python, and memory management rules for PyO3 bindings.
  - relevant sections: GIL Management, Error Propagation, Memory Ownership
- **knowledge:spec-to-code/spec-model.md**
  - summary: Defines how specs map to code, specifically how Requirement+ acts as a blueprint for generating tests with traceability.
  - relevant sections: Requirement Plus - Test Verification, Spec Catalog

## Patterns

- **PyO3 Call Flow** (source: orbit/bridge-internals.md)
  - Acquire GIL, convert args, release GIL for async/blocking work, execute, reacquire GIL, convert result.
- **Requirement Plus Traceability** (source: spec-to-code/spec-model.md)
  - N:M traceability between requirements, scenarios (test functions), and modules. Uses docrefs for code mapping.
- **Directive-based Fixture Testing** (source: fixture_tests.rs)
  - File-based test harness using datatest-stable to run .py files with # RUN: directives.

## Pitfalls

- GIL Deadlocks: Holding GIL while waiting for threads that need it.
- Over-broad Testing: Full CPython suite is too large; need curated syntax subset.
