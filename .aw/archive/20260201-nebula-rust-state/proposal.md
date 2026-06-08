---
id: nebula-rust-state
type: proposal
version: 1
created_at: 2026-02-01T14:25:41.390491+00:00
updated_at: 2026-02-01T14:25:41.390491+00:00
author: mcp
status: proposed
iteration: 1
summary: "Move State Management (COW change tracking) to Rust with BSON storage"
history:
  - timestamp: 2026-02-01T14:25:41.390491+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-01T14:30:10.045068+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-01T14:30:20.264151+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 5
  new_files: 2
affected_specs:
  - id: nebula-rust-state-spec
    path: specs/nebula-rust-state-spec.md
    depends: []
  - id: nebula-rust-state-pyo3-spec
    path: specs/nebula-rust-state-pyo3-spec.md
    depends: []---

<proposal>

# Change: nebula-rust-state

## Summary

Move State Management (COW change tracking) to Rust with BSON storage

## Why

Moving change tracking to Rust improves performance (estimated 10x faster) and reduces memory overhead for large documents. Storing original values in BSON ensures consistency with the MongoDB driver and prepares the codebase for further Rust-native document logic. Full rollback support is required to support transaction-like semantics.

## What Changes

- Implement StateTracker struct in crates/cclab-nebula/src/state.rs using bson::Document for original value storage.
- Support field-level change tracking where nested changes mark the parent field as dirty (field-level granularity).
- Add track_change, get_changes, rollback, is_modified, has_changed, reset, and get_all_original_data methods to Rust StateTracker.
- Expose Rust StateTracker to Python via PyO3 bindings in crates/cclab-nucleus/src/nebula/state.rs.
- Update Python StateTracker in python/cclab/nebula/state.py to be a thin wrapper around the Rust implementation.

## Impact

- **Scope**: minor
- **Affected Files**: ~5
- **New Files**: ~2
- Affected specs:
  - `nebula-rust-state-spec` (no dependencies)
  - `nebula-rust-state-pyo3-spec` (no dependencies)
- Affected code: `crates/cclab-nebula/src/state.rs`, `crates/cclab-nebula/src/lib.rs`, `crates/cclab-nucleus/src/nebula/mod.rs`, `crates/cclab-nucleus/src/nebula/state.rs`, `python/cclab/nebula/state.py`

</proposal>
