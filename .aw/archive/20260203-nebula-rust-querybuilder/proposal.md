---
id: nebula-rust-querybuilder
type: proposal
version: 1
created_at: 2026-02-01T07:10:59.189141+00:00
updated_at: 2026-02-01T07:10:59.189141+00:00
author: mcp
status: proposed
iteration: 1
summary: "Move Nebula QueryBuilder and QueryExpr core logic to Rust with clone-based chainable API."
history:
  - timestamp: 2026-02-01T07:10:59.189141+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-01T07:11:51.229031+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-01T07:12:06.291538+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 6
  new_files: 0
affected_specs:
  - id: nebula-rust-querybuilder-spec
    path: specs/nebula-rust-querybuilder-spec.md
    depends: []
  - id: querybuilder-types-spec
    path: specs/querybuilder-types-spec.md
    depends: []
  - id: querybuilder-pyo3-spec
    path: specs/querybuilder-pyo3-spec.md
    depends: []---

<proposal>

# Change: nebula-rust-querybuilder

## Summary

Move Nebula QueryBuilder and QueryExpr core logic to Rust with clone-based chainable API.

## Why

Moving query construction and execution to Rust significantly improves performance by reducing GIL contention and centralizing validation logic. A clone-based chainable API provides a safer, immutable pattern that aligns with Rust conventions and avoids race conditions in multi-threaded Python environments.

## What Changes

- Implement QueryExpr in Rust to represent MongoDB conditions with support for logical AND/OR operations.
- Refactor QueryBuilder in cclab-nebula to use a clone-based chainable API for filter, sort, skip, limit, and projection.
- Expose RustQueryExpr and RustQueryBuilder as PyO3 classes in cclab-nucleus to bridge Python and Rust.
- Update Python FieldProxy and QueryBuilder to delegate core operations to the new Rust implementation.

## Impact

- **Scope**: minor
- **Affected Files**: ~6
- **New Files**: ~0
- Affected specs:
  - `nebula-rust-querybuilder-spec` (no dependencies)
  - `querybuilder-types-spec` (no dependencies)
  - `querybuilder-pyo3-spec` (no dependencies)

</proposal>
