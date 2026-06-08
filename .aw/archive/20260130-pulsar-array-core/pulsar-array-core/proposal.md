---
id: pulsar-array-core
type: proposal
version: 1
created_at: 2026-01-30T03:34:18.472954+00:00
updated_at: 2026-01-30T03:34:18.472954+00:00
author: mcp
status: proposed
iteration: 1
summary: "Foundational N-dimensional array crate for the Pulsar ecosystem, providing a pure-Rust, zero-dependency alternative to NumPy."
history:
  - timestamp: 2026-01-30T03:34:18.472954+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-30T03:35:32.537234+00:00
    agent: "unknown"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-30T03:35:51.531057+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
  - timestamp: 2026-01-30T03:38:59.070188+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-30T03:39:20.554776+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: major
  affected_files: 5
  new_files: 5
affected_specs:
  - id: pulsar-array-core-design
    path: specs/pulsar-array-core-design.md
    depends: []---

<proposal>

# Change: pulsar-array-core

## Summary

Foundational N-dimensional array crate for the Pulsar ecosystem, providing a pure-Rust, zero-dependency alternative to NumPy.

## Why

To build high-performance ML and data science tools (similar to pandas and scipy) within the cclab ecosystem, a robust N-dimensional array foundation is required. By implementing this in pure Rust without external dependencies, we ensure maximum performance control, long-term maintainability, and zero supply-chain risk for the core data structures.

## What Changes

- Create new crate `crates/cclab-pulsar-array-core`
- Implement `NdArray` structure for multi-dimensional data storage
- Design a flexible DType system supporting core numeric types
- Implement NumPy-style broadcasting, slicing, and indexing logic from scratch
- Provide basic element-wise mathematical operations (+, -, *, /)
- Update `cclab/specs/crate-map.md` to include the Pulsar ecosystem

## Impact

- **Scope**: major
- **Affected Files**: ~5
- **New Files**: ~5
- Affected specs:
  - `pulsar-array-core-design` (no dependencies)
- Affected code: `crates/cclab-pulsar-array-core`

</proposal>
