---
id: nebula-rust-link-fetch
type: proposal
version: 1
created_at: 2026-02-01T10:46:50.033174+00:00
updated_at: 2026-02-01T10:46:50.033174+00:00
author: mcp
status: proposed
iteration: 1
summary: "Move Batched Link Fetching to Rust with full recursive depth support"
history:
  - timestamp: 2026-02-01T10:46:50.033174+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 6
  new_files: 2
---

<proposal>

# Change: nebula-rust-link-fetch

## Summary

Move Batched Link Fetching to Rust with full recursive depth support

## Why

Link fetching is performance-critical in Nebula ORM. Currently the batching logic is implemented in Python, causing multiple Python-Rust roundtrips and inefficient HashMap operations. Moving the entire pipeline to Rust will reduce latency, enable better memory management, and support recursive depth fetching more efficiently.

## What Changes

- Implement LinkField, LinkType, and LinkRef types in cclab-nebula crate
- Implement fetch_links_batched async function with ref collection, batch query, and distribution
- Support recursive depth fetching for nested links
- Expose PyO3 bindings for Python thin wrapper integration

## Impact

- **Scope**: minor
- **Affected Files**: ~6
- **New Files**: ~2
- Affected code: `crates/cclab-nebula/src/link.rs`, `crates/cclab-nucleus/src/nebula/link.rs`, `crates/cclab-nucleus/src/nebula/mod.rs`, `python/cclab/nebula/document.py`

</proposal>
