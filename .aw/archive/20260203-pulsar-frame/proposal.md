---
id: pulsar-frame
type: proposal
version: 1
created_at: 2026-01-30T06:34:00.735406+00:00
updated_at: 2026-01-30T06:34:00.735406+00:00
author: mcp
status: proposed
iteration: 1
summary: "Create cclab-pulsar-frame: Pure Rust DataFrame library with pandas-compatible API using pulsar-array"
history:
  - timestamp: 2026-01-30T06:34:00.735406+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-30T06:44:07.379203+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-30T06:44:22.040871+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 20
  new_files: 20
affected_specs:
  - id: pulsar-frame-core
    path: specs/pulsar-frame-core.md
    depends: []
  - id: pulsar-frame-io
    path: specs/pulsar-frame-io.md
    depends: [pulsar-frame-core]
  - id: pulsar-frame-ops
    path: specs/pulsar-frame-ops.md
    depends: [pulsar-frame-core]
  - id: pulsar-frame-shield
    path: specs/pulsar-frame-shield.md
    depends: [pulsar-frame-core]---

<proposal>

# Change: pulsar-frame

## Summary

Create cclab-pulsar-frame: Pure Rust DataFrame library with pandas-compatible API using pulsar-array

## Why

To provide a high-performance, type-safe data manipulation library native to the cclab ecosystem. It leverages cclab-pulsar-array for efficient storage and cclab-shield for robust schema validation, aiming to offer a familiar pandas-like API for Rust developers. This fills the gap for a DataFrame library that integrates seamlessly with existing cclab tools.

## What Changes

- Create new crate crates/cclab-pulsar-frame
- Implement Series struct wrapping cclab-pulsar-array::NdArray
- Implement DataFrame struct managing columns of Series
- Implement Index system (Range, Int, String)
- Implement loc and iloc indexing traits
- Implement IO modules for CSV, JSON, and Parquet
- Integrate cclab-shield for schema definition and validation
- Implement GroupBy and Join operations

## Impact

- **Scope**: minor
- **Affected Files**: ~20
- **New Files**: ~20
- Affected specs:
  - `pulsar-frame-core` (no dependencies)
  - `pulsar-frame-io` → depends on: `pulsar-frame-core`
  - `pulsar-frame-ops` → depends on: `pulsar-frame-core`
  - `pulsar-frame-shield` → depends on: `pulsar-frame-core`
- Affected code: `crates/cclab-pulsar-frame`
- **Breaking Changes**: None (new crate)

</proposal>
