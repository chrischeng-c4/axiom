---
id: improve-titan-maturity
type: proposal
version: 1
created_at: 2026-01-28T08:03:43.048011+00:00
updated_at: 2026-01-28T08:03:43.048011+00:00
author: mcp
status: proposed
iteration: 1
summary: "Upgrade cclab-titan to 95% maturity with Multi-Dialect support, Rust-based Unit of Work, Hook system, and Hybrid Properties."
history:
  - timestamp: 2026-01-28T08:03:43.048011+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-28T08:11:46.685474+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_proposal"
    action: "created"
    duration_secs: 747.68
  - timestamp: 2026-01-28T08:12:56.797079+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 70.11
  - timestamp: 2026-01-28T08:17:07.997571+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 251.19
  - timestamp: 2026-01-28T08:18:16.240809+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 68.24
impact:
  scope: major
  affected_files: 25
  new_files: 8
affected_specs:
  - id: dialect-abstraction
    path: specs/dialect-abstraction.md
    depends: []
  - id: session-unit-of-work
    path: specs/session-unit-of-work.md
    depends: [dialect-abstraction]
  - id: hook-system
    path: specs/hook-system.md
    depends: [session-unit-of-work]
  - id: hybrid-properties
    path: specs/hybrid-properties.md
    depends: [dialect-abstraction]---

<proposal>

# Change: improve-titan-maturity

## Summary

Upgrade cclab-titan to 95% maturity with Multi-Dialect support, Rust-based Unit of Work, Hook system, and Hybrid Properties.

## Why

cclab-titan is currently 60% mature and tied to PostgreSQL. To become a viable SQLAlchemy alternative, it needs multi-dialect support and a high-performance session management layer in Rust. Porting the Unit of Work to Rust aligns with the core principle of 'Zero Python byte handling' and improves consistency and performance across the library.

## What Changes

- Implement Dialect trait and Sqlite/MySQL support in cclab-titan (Rust)
- Port Session and Unit of Work (identity map, dirty tracking) from Python to Rust
- Add lifecycle events/hooks (before_insert, after_update, etc.) to Session
- Implement Hybrid Properties (SQL-expression attributes) in QueryBuilder
- Enhance integration tests for migration rollbacks and transaction isolation across all dialects
- Expose new Rust features to Python via cclab-nucleus (PyO3)
- Update documentation with Active Record vs Data Mapper guide and CTE examples

## Impact

- **Scope**: major
- **Affected Files**: ~25
- **New Files**: ~8
- Affected specs:
  - `dialect-abstraction` (no dependencies)
  - `session-unit-of-work` → depends on: `dialect-abstraction`
  - `hook-system` → depends on: `session-unit-of-work`
  - `hybrid-properties` → depends on: `dialect-abstraction`
- Affected code: `crates/cclab-titan/src/query/*`, `crates/cclab-titan/src/schema.rs`, `crates/cclab-titan/src/executor.rs`, `crates/cclab-titan/src/connection.rs`, `crates/cclab-titan/src/types.rs`, `crates/cclab-nucleus/src/postgres/*`, `python/cclab/titan/session.py`
- **Breaking Changes**: Moving session logic to Rust might change internal PyO3 bridge APIs. Multi-dialect support introduces new configuration options for database URIs.

</proposal>

<review iteration="1" reviewer="gemini" status="needs_revision">
## Summary
The proposal for `improve-titan-maturity` is architecturally sound and aligns with the exploration findings. However, the supporting specifications and task list fall short of the mandatory quality standards required for spec-to-code generation.

## Issues

### HIGH Severity
1. **Insufficient Acceptance Scenarios**: All 5 specifications (`dialect-abstraction`, `hook-system`, `hybrid-properties`, `session-unit-of-work`, `test-doc-gaps`) only include 2 acceptance scenarios. The mandatory minimum is 3 scenarios per spec.
2. **Incorrect Task File Paths**: `tasks.md` uses generic file paths (e.g., `src/logic/dialect-abstraction.rs`) which do not match the project structure identified in the exploration and proposal (`crates/cclab-titan/src/...`). This will cause the implementation phase to fail or create files in the wrong locations.

### MEDIUM Severity
3. **Missing Interface Definitions**: Major architectural components like the `Dialect` trait, `Session`, and `UnitOfWork` lack pseudo-code interface definitions (`FUNCTION name(params) -> Result`) in their respective specs. This leaves the "HOW" too ambiguous for implementation.
4. **Missing Data Models**: `session-unit-of-work.md` and `hook-system.md` should include JSON schemas to define the structure of the Identity Map, Dirty Tracker state, and Hook Registry.
5. **Incoherent Spec Type**: `test-doc-gaps.md` is typed as `algorithm` but contains requirements for integration tests and documentation guides. This should be refactored or moved into the testing layer of `tasks.md`.

### LOW Severity
6. **Missing Task Layers**: `tasks.md` skips "1. Data Layer" and "3. Integration Layer", which are part of the required task structure.

## Verdict
NEEDS_REVISION

## Next Steps
1. Add at least one more scenario (error path or edge case) to every specification.
2. Update all file paths in `tasks.md` to reflect the actual project structure (`crates/cclab-titan/...`).
3. Add pseudo-code interface definitions to `dialect-abstraction.md`, `session-unit-of-work.md`, and `hook-system.md`.
4. Add JSON schemas for stateful components in `session-unit-of-work.md`.
5. Refactor `test-doc-gaps.md` or redistribute its requirements into more appropriate locations (e.g., testing tasks).
</review>
