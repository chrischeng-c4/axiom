---
id: nebula-thin-wrapper
type: proposal
version: 1
created_at: 2026-02-03T09:31:43.633592+00:00
updated_at: 2026-02-03T09:31:43.633592+00:00
author: mcp
status: proposed
iteration: 1
summary: "Migrate cclab.nebula Python logic to Rust thin wrapper architecture with full parity"
history:
  - timestamp: 2026-02-03T09:31:43.633592+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-03T09:35:17.487207+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-03T09:35:39.604902+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: major
  affected_files: 6
  new_files: 0
affected_specs:
  - id: query-builder
    path: specs/query-builder.md
    depends: []
  - id: link-fetching
    path: specs/link-fetching.md
    depends: []
  - id: bulk-write
    path: specs/bulk-write.md
    depends: []
  - id: state-management
    path: specs/state-management.md
    depends: []---

<proposal>

# Change: nebula-thin-wrapper

## Summary

Migrate cclab.nebula Python logic to Rust thin wrapper architecture with full parity

## Why

To improve performance and leverage Rust's type safety and efficiency for core database operations, specifically targeting bulk writes, aggregation, query building, link fetching, and state management as per issues #79, #80, #97, #99, #100. Ensuring full feature parity (writes/updates) is critical for a drop-in replacement.

## What Changes

- Modify python/cclab/nebula/query.py to use Rust PyQueryBuilder for reads and writes
- Implement update, delete, upsert, and atomic operators in Rust PyQueryBuilder
- Modify python/cclab/nebula/state.py to use Rust PyStateTracker
- Modify python/cclab/nebula/links.py to use Rust fetch_links_batched
- Modify crates/cclab-nebula/src/pyo3_bindings/document.rs to expose bulk_write
- Update python/cclab/nebula/bulk.py to delegate to Rust bulk_write

## Impact

- **Scope**: major
- **Affected Files**: ~6
- **New Files**: ~0
- Affected specs:
  - `query-builder` (no dependencies)
  - `link-fetching` (no dependencies)
  - `bulk-write` (no dependencies)
  - `state-management` (no dependencies)
- Affected code: `python/cclab/nebula/query.py`, `python/cclab/nebula/state.py`, `python/cclab/nebula/links.py`, `python/cclab/nebula/bulk.py`, `crates/cclab-nebula/src/pyo3_bindings/document.rs`, `crates/cclab-nebula/src/pyo3_bindings/query.rs`
- **Breaking Changes**: None, internal logic migration only. APIs should remain compatible.

</proposal>
