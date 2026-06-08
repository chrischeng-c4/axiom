---
id: orbit-p2-enhancements
type: proposal
version: 1
created_at: 2026-02-05T13:34:13.743777+00:00
updated_at: 2026-02-05T13:34:13.743777+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add testing infrastructure, benchmarks, and debug API for orbit event loop"
history:
  - timestamp: 2026-02-05T13:34:13.743777+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 8
  new_files: 4
affected_specs:
  - id: debug-api
    path: specs/debug-api.md
    depends: []
  - id: integration-tests
    path: specs/integration-tests.md
    depends: [debug-api]
  - id: benchmarks
    path: specs/benchmarks.md
    depends: []
  - id: stress-tests
    path: specs/stress-tests.md
    depends: [integration-tests]
  - id: bridge-docs
    path: specs/bridge-docs.md
    depends: [debug-api]
  - id: tuning-guide
    path: specs/tuning-guide.md
    depends: [benchmarks]
---

<proposal>

# Change: orbit-p2-enhancements

## Summary

Add testing infrastructure, benchmarks, and debug API for orbit event loop

## Why

The orbit crate provides a high-performance asyncio event loop implementation but lacks comprehensive testing and developer tooling. Currently:

1. **No integration tests**: Only unit tests exist in individual modules. There are no end-to-end tests validating the complete asyncio protocol implementation, making it risky to refactor or add features.

2. **No performance benchmarks**: Without benchmarks comparing against uvloop and asyncio, we cannot quantify performance gains or detect regressions. Users have no data to justify adopting orbit.

3. **Incomplete debug mode**: The `DebugMonitor` class exists in Rust with full statistics tracking, but `get_debug_stats()` is not exposed to Python. Users can only toggle debug on/off but cannot inspect loop statistics programmatically.

4. **Documentation gaps**: Knowledge base has basic docs for bridge internals and tuning, but they lack concrete examples, benchmark data, and detailed architecture diagrams.

This change addresses GitHub issues #69-#74 (all P2 priority), establishing orbit as a production-ready event loop with proper testing, observability, and documentation.

## What Changes

- Expose DebugMonitor statistics to Python via get_debug_stats() method on PyLoop
- Create Rust integration tests for event loop protocol compliance (TCP, UDP, timers, signals)
- Add Criterion benchmarks comparing orbit vs uvloop vs asyncio for timers, I/O, and task throughput
- Create stress tests validating 10k concurrent connections and task cancellation under load
- Enhance bridge-internals.md with waker implementation details and error handling flow
- Update performance-tuning.md with benchmark results and concrete configuration examples

## Impact

- **Scope**: minor
- **Affected Files**: ~8
- **New Files**: ~4
- Affected specs:
  - `debug-api` (no dependencies)
  - `integration-tests` → depends on: `debug-api`
  - `benchmarks` (no dependencies)
  - `stress-tests` → depends on: `integration-tests`
  - `bridge-docs` → depends on: `debug-api`
  - `tuning-guide` → depends on: `benchmarks`
- Affected code: `crates/cclab-orbit/src/loop_impl.rs`, `crates/cclab-orbit/src/debug.rs`, `crates/cclab-orbit/tests/integration_tests.rs`, `crates/cclab-orbit/benches/benchmarks.rs`, `crates/cclab-orbit/tests/stress_tests.rs`, `crates/cclab-orbit/Cargo.toml`, `cclab/knowledge/orbit/bridge-internals.md`, `cclab/knowledge/orbit/performance-tuning.md`

</proposal>
