---
id: pulsar-phase2
type: proposal
version: 1
created_at: 2026-01-31T09:35:34.534008+00:00
updated_at: 2026-01-31T09:35:34.534008+00:00
author: mcp
status: proposed
iteration: 1
summary: "Phase 2: Expanded statistical analysis and missing value handling in Pulsar ecosystem."
history:
  - timestamp: 2026-01-31T09:35:34.534008+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T09:35:57.315339+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T09:36:07.396705+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 20
  new_files: 8
affected_specs:
  - id: pulsar-array-ext
    path: specs/pulsar-array-ext.md
    depends: []
  - id: pulsar-frame-ext
    path: specs/pulsar-frame-ext.md
    depends: []
  - id: pulsar-stats
    path: specs/pulsar-stats.md
    depends: []---

<proposal>

# Change: pulsar-phase2

## Summary

Phase 2: Expanded statistical analysis and missing value handling in Pulsar ecosystem.

## Why

To achieve 50%+ API coverage of NumPy/Pandas/SciPy, specifically targeting high-priority areas like data cleaning (missing values) and statistical analysis as identified in recent clarifications.

## What Changes

- pulsar-array: Expansion of statistical functions (median, mode, skew, kurtosis) and complex number support.
- pulsar-frame: Implementation of missing value handling (fillna, dropna, isna, interpolate) and advanced GroupBy/Join.
- pulsar-stats: Introduction of a comprehensive statistical module covering probability distributions and hypothesis testing (scipy.stats equivalent).

## Impact

- **Scope**: minor
- **Affected Files**: ~20
- **New Files**: ~8
- Affected specs:
  - `pulsar-array-ext` (no dependencies)
  - `pulsar-frame-ext` (no dependencies)
  - `pulsar-stats` (no dependencies)
- Affected code: `crates/cclab-pulsar/src/array/stats.rs`, `crates/cclab-pulsar/src/array/linalg.rs`, `crates/cclab-pulsar/src/array/dtype.rs`, `crates/cclab-pulsar/src/frame/ops/missing.rs`, `crates/cclab-pulsar/src/frame/ops/mod.rs`, `crates/cclab-pulsar/src/stats/mod.rs`

</proposal>
