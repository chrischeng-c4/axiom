---
id: probe-multilang
type: proposal
version: 1
created_at: 2026-02-01T13:01:19.504571+00:00
updated_at: 2026-02-01T13:01:19.504571+00:00
author: mcp
status: proposed
iteration: 1
summary: "Extend cclab-probe to support Rust and TypeScript testing while unifying multi-language reporting with existing Python support."
history:
  - timestamp: 2026-02-01T13:01:19.504571+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-01T13:01:24.672084+00:00
    agent: "codex:deep"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-01T13:01:38.024472+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 15
  new_files: 8
affected_specs:
  - id: rust-runner-integration-spec
    path: specs/rust-runner-integration-spec.md
    depends: []
  - id: typescript-custom-runner-spec
    path: specs/typescript-custom-runner-spec.md
    depends: []
  - id: multi-lang-unified-reporting-spec
    path: specs/multi-lang-unified-reporting-spec.md
    depends: [rust-runner-integration-spec, typescript-custom-runner-spec]---

<proposal>

# Change: probe-multilang

## Summary

Extend cclab-probe to support Rust and TypeScript testing while unifying multi-language reporting with existing Python support.

## Why

cclab-probe currently focuses on Python, but the core platform is Rust and many services are TypeScript. Adding Rust integration (via cargo) and a custom TypeScript runner enables consistent testing, profiling, and security coverage across the stack. A unified report plus per-language details ensures cross-language visibility without losing language-specific metrics.

## What Changes

- Add a language-aware model (Language enum + TestMeta/TestResult updates) to tag results from Python, Rust, and TypeScript.
- Extend discovery to detect Rust (Cargo.toml) and TypeScript (package.json/tsconfig.json) projects alongside existing Python detection.
- Integrate Rust execution via cargo test --message-format=json, cargo bench, and cargo-fuzz/cargo-audit for security workflows.
- Implement a custom TypeScript runner that supports multiple harness styles (e.g., jest/vitest/mocha), parallel execution, and V8 profiling metrics.
- Expand reporting to produce a unified cross-language summary plus per-language sections with toolchain metadata (rustc/node/python) and language-specific metrics.
- Broaden performance/security modules to emit shared metrics (latency, throughput, memory) and language-specific deep diagnostics (Rust flamegraph, V8 profiler, Python cProfile/GIL).

## Impact

- **Scope**: minor
- **Affected Files**: ~15
- **New Files**: ~8
- Affected specs:
  - `rust-runner-integration-spec` (no dependencies)
  - `typescript-custom-runner-spec` (no dependencies)
  - `multi-lang-unified-reporting-spec` → depends on: `rust-runner-integration-spec`, `typescript-custom-runner-spec`
- Affected code: `crates/cclab-probe/src/discovery.rs`, `crates/cclab-probe/src/runner.rs`, `crates/cclab-probe/src/reporter.rs`, `crates/cclab-probe/src/performance/mod.rs`, `crates/cclab-probe/src/security/mod.rs`

</proposal>
