---
id: nebula-rust-aggregate
type: proposal
version: 1
created_at: 2026-01-31T14:10:07.432418+00:00
updated_at: 2026-01-31T14:10:07.432418+00:00
author: mcp
status: proposed
iteration: 1
summary: "Move MongoDB aggregation pipeline construction and execution into Rust, exposing a fluent AggregationBuilder to Python while keeping the Python layer thin."
history:
  - timestamp: 2026-01-31T14:10:07.432418+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T14:10:11.421833+00:00
    agent: "codex:deep"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T14:10:21.523729+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 12
  new_files: 0---

<proposal>

# Change: nebula-rust-aggregate

## Summary

Move MongoDB aggregation pipeline construction and execution into Rust, exposing a fluent AggregationBuilder to Python while keeping the Python layer thin.

## Why

Nebula aggregation is currently assembled and executed from Python, and the Rust backend lacks an aggregate entrypoint even though the Python API expects one. This keeps validation and security checks in Python and forces the Python layer to build pipelines for helpers like avg/sum/min/max/count. Moving pipeline building and execution into Rust centralizes validation, improves performance, enables consistent security enforcement, and aligns with the requested $match + $group scope.

## What Changes

- Implement a Rust aggregation builder and pipeline executor in `crates/cclab-nebula` supporting `$match` and `$group` plus accumulator helpers (avg/sum/min/max/count).
- Add Rust-side pipeline validation to accept only allowed stages/operators and reject dangerous operators (e.g., `$accumulator`, `$where`) to satisfy security tests.
- Expose the builder and aggregate entrypoint through PyO3 bindings in `crates/cclab-nucleus`, adding new classes/methods on the Nebula module.
- Wire Python `AggregationBuilder` and helper methods to the Rust builder/aggregate implementation while keeping `Document.aggregate(pipeline)` working.
- Update Python type stubs/docs as needed to reflect the Rust-backed aggregation APIs.
- Add/adjust tests for Rust builder construction/validation and Python aggregation execution with `$match` + `$group`.

## Impact

- **Scope**: minor
- **Affected Files**: ~12
- **New Files**: ~0
- Affected code: `crates/cclab-nebula/src/lib.rs`, `crates/cclab-nebula/src/query.rs or new aggregation module`, `crates/cclab-nebula/src/validation.rs`, `crates/cclab-nucleus/src/nebula/mod.rs`, `crates/cclab-nucleus/src/nebula/document.rs`, `python/cclab/nebula/_engine.py`, `python/cclab/nebula/query.py`, `python/cclab/nebula/document.py`, `python/cclab/nebula/mongodb.pyi`, `python/tests/mongo/unit/test_security.py (or new aggregation tests)`
- **Breaking Changes**: None expected; existing `Document.aggregate(pipeline)` remains supported and helper signatures stay the same. Rust validation may surface errors earlier for invalid pipelines.

</proposal>
