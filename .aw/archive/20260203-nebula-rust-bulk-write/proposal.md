---
id: nebula-rust-bulk-write
type: proposal
version: 1
created_at: 2026-01-31T10:43:03.253967+00:00
updated_at: 2026-01-31T10:43:03.253967+00:00
author: mcp
status: proposed
iteration: 1
summary: "Migrate nebula bulk_write core logic to Rust using Enum-based design and FromPyObject conversion."
history:
  - timestamp: 2026-01-31T10:43:03.253967+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 4
  new_files: 1
affected_specs:
  - id: nebula-bulk-write-rust
    path: specs/nebula-bulk-write-rust.md
    depends: []
---

<proposal>

# Change: nebula-rust-bulk-write

## Summary

Migrate nebula bulk_write core logic to Rust using Enum-based design and FromPyObject conversion.

## Why

Current bulk write implementation relies on a Python-side fallback that executes operations one-by-one, which is inefficient. Moving this logic to Rust allows for faster BSON serialization, better GIL management, and a more robust type-safe design using Rust Enums. Implementing `FromPyObject` simplifies the bridge between Python and Rust.

## What Changes

- Implement `bulk_write` static method in `RustDocument` (Rust backend).
- Create `BulkOperation` enum in Rust with `FromPyObject` trait implementation for automatic Python-to-Rust conversion.
- Refactor Python `BulkOperation` classes in `bulk.py` to produce the updated format required by the Rust enum.
- Remove Python-side fallback in `_engine.py` and ensure direct delegation to the Rust implementation.

## Impact

- **Scope**: minor
- **Affected Files**: ~4
- **New Files**: ~1
- Affected specs:
  - `nebula-bulk-write-rust` (no dependencies)
- Affected code: `crates/cclab-nucleus/src/nebula/document.rs`, `crates/cclab-nucleus/src/nebula/types.rs`, `python/cclab/nebula/bulk.py`, `python/cclab/nebula/_engine.py`
- **Breaking Changes**: The internal dictionary format passed from Python to Rust for bulk operations will be updated to match the new Rust Enum structure. This simplifies the backend but changes the expected input for the `bulk_write` method.

</proposal>
