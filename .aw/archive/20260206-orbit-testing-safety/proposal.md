---
id: orbit-testing-safety
type: proposal
version: 1
created_at: 2026-02-05T16:12:44.548543+00:00
updated_at: 2026-02-05T16:12:44.548543+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add fuzz testing infrastructure and Miri CI integration for orbit crate"
history:
  - timestamp: 2026-02-05T16:12:44.548543+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: patch
  affected_files: 3
  new_files: 5
affected_specs:
  - id: fuzz-targets
    path: specs/fuzz-targets.md
    depends: []
  - id: miri-ci
    path: specs/miri-ci.md
    depends: []
---

<proposal>

# Change: orbit-testing-safety

## Summary

Add fuzz testing infrastructure and Miri CI integration for orbit crate

## Why

The orbit crate provides a high-performance asyncio event loop for Python, handling critical operations like timer management, waker signaling, and handle lifecycle. While the crate enforces `#![forbid(unsafe_code)]`, fuzz testing is essential for discovering edge cases in timer wheel operations, waker state machines, and concurrent access patterns. Miri integration validates atomic ordering correctness and catches subtle memory model violations that could cause issues on different CPU architectures.

## What Changes

- Add cargo-fuzz infrastructure with fuzz targets for TimerWheel, PythonWaker, and Handle operations
- Create fuzz targets that isolate pure-Rust logic from PyO3 dependencies
- Add Miri test configuration to CI workflow for atomic ordering validation
- Document fuzz testing procedures and corpus management

## Impact

- **Scope**: patch
- **Affected Files**: ~3
- **New Files**: ~5
- Affected specs:
  - `fuzz-targets` (no dependencies)
  - `miri-ci` (no dependencies)
- Affected code: `crates/cclab-orbit/fuzz/`, `.github/workflows/`

</proposal>
