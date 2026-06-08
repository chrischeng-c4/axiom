---
id: 178-grid-db-refactor
type: proposal
version: 1
created_at: 2026-02-05T04:42:49.543822+00:00
updated_at: 2026-02-05T04:42:49.543822+00:00
author: mcp
status: proposed
iteration: 1
summary: "Refactor grid-db persistence with shared WAL crate, Morton cell storage, and yrs snapshot CRDT payloads"
history:
  - timestamp: 2026-02-05T04:42:49.543822+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-05T04:42:54.747885+00:00
    agent: "codex:deep"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-05T04:43:07.767008+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 26
  new_files: 8
affected_specs:
  - id: grid-db-architecture
    path: specs/grid-db-architecture.md
    depends: []---

<proposal>

# Change: 178-grid-db-refactor

## Summary

Refactor grid-db persistence with shared WAL crate, Morton cell storage, and yrs snapshot CRDT payloads

## Why

`cclab-grid-db` is largely stubbed and currently cannot persist or query cells. The CRDT module’s custom LWW operations diverge from the `yrs`-based collaboration already used by `cclab-grid-server`. WAL logic is embedded inside `cclab-ion`; extracting a shared `cclab-wal` crate avoids duplication and enables consistent durability and recovery across storage engines.

## What Changes

- Extract WAL logic into a new `cclab-wal` crate (writer/reader, format, rotation, recovery) and update `cclab-ion` to use it
- Implement `cclab-grid-db` storage core: Morton encode/decode, rectangular range mapping, `CellStore` CRUD, WAL-backed durability, and stats/flush paths
- Refactor `cclab-grid-db` CRDT module to store `yrs` updates/snapshots rather than custom LWW operations while keeping the module boundary intact
- Align public APIs and docs across `cclab-grid-db` and `cclab-ion` for the shared WAL integration
- Add focused unit/integration tests for Morton round-trip, WAL replay, and cell persistence/range queries

## Impact

- **Scope**: minor
- **Affected Files**: ~26
- **New Files**: ~8
- Affected specs:
  - `grid-db-architecture` (no dependencies)
- Affected code: `crates/cclab-grid-db/src/lib.rs`, `crates/cclab-grid-db/src/storage/mod.rs`, `crates/cclab-grid-db/src/storage/cell_store.rs`, `crates/cclab-grid-db/src/storage/morton.rs`, `crates/cclab-grid-db/src/storage/wal.rs`, `crates/cclab-grid-db/src/query/mod.rs`, `crates/cclab-grid-db/src/query/range.rs`, `crates/cclab-grid-db/src/crdt/mod.rs`, `crates/cclab-grid-db/src/crdt/operations.rs`, `crates/cclab-grid-db/Cargo.toml`, `crates/cclab-ion/src/persistence/wal.rs`, `crates/cclab-ion/src/persistence/format.rs`, `crates/cclab-ion/src/persistence/recovery.rs`, `crates/cclab-ion/src/persistence/handle.rs`, `crates/cclab-ion/Cargo.toml`, `crates/cclab-grid-server/src/db/mod.rs`, `Cargo.toml`, `crates/cclab-wal/Cargo.toml`, `crates/cclab-wal/src/lib.rs`
- **Breaking Changes**: CRDT payload format switches from custom LWW ops to `yrs` updates/snapshots; `cclab-ion` persistence modules move to use the shared `cclab-wal` crate, which may require import path updates downstream.

</proposal>
