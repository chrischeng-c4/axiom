---
id: rust-querybuilder-extend
type: proposal
version: 1
created_at: 2026-01-31T10:10:51.707688+00:00
updated_at: 2026-01-31T10:10:51.707688+00:00
author: mcp
status: proposed
iteration: 1
summary: "Extend Rust QueryBuilder to support all Python features with a fluent API."
history:
  - timestamp: 2026-01-31T10:10:51.707688+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T10:13:19.570536+00:00
    agent: "unknown"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T10:13:40.480943+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
  - timestamp: 2026-01-31T10:16:25.739654+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T10:16:41.181259+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 10
  new_files: 0
affected_specs:
  - id: cclab-titan/01-core-engine/10-components
    path: specs/cclab-titan/01-core-engine/10-components.md
    depends: []
  - id: cclab-titan/01-core-engine/30-implementation-details
    path: specs/cclab-titan/01-core-engine/30-implementation-details.md
    depends: []---

<proposal>

# Change: rust-querybuilder-extend

## Summary

Extend Rust QueryBuilder to support all Python features with a fluent API.

## Why

To achieve feature parity between the Rust and Python QueryBuilder implementations. This allows the Python cclab.titan.query to be refactored into a thin wrapper over the Rust core, improving performance, maintainability, and consistency across dialects.

## What Changes

- Add aggregate convenience methods: count_agg, sum, avg, min, max, count_column, count_distinct.
- Add grouping and HAVING clause convenience methods: having_sum, having_count, etc.
- Implement window function convenience methods: row_number, rank, lag, lead, etc.
- Add CTE support with from_cte static method and improve with_cte.
- Integrate JSONB operators: jsonb_contains, jsonb_has_key, etc.
- Add RETURNING clause support for mutations (INSERT/UPDATE/DELETE).
- Refactor builder methods to consistently use the mutable self pattern for chaining.

## Impact

- **Scope**: minor
- **Affected Files**: ~10
- **New Files**: ~0
- Affected specs:
  - `cclab-titan/01-core-engine/10-components` (no dependencies)
  - `cclab-titan/01-core-engine/30-implementation-details` (no dependencies)
- Affected code: `crates/cclab-titan/src/query/builder.rs`, `crates/cclab-titan/src/query/select.rs`, `crates/cclab-titan/src/query/modify.rs`, `crates/cclab-titan/src/query/types.rs`, `crates/cclab-titan/src/query/window.rs`

</proposal>
