---
id: cclab-taipan
type: proposal
version: 1
created_at: 2026-02-12T07:40:53.405201+00:00
updated_at: 2026-02-12T07:40:53.405201+00:00
author: mcp
status: proposed
iteration: 1
summary: "Implement Taipan high-performance compiler v0.1 with Cranelift backend and unified CLI integration."
history:
  - timestamp: 2026-02-12T07:40:53.405201+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 20
  new_files: 0
affected_specs:
  - id: cli-architecture
    path: specs/cli-architecture.md
    depends: []
  - id: aurora-codegen-system
    path: specs/aurora-codegen-system.md
    depends: []
  - id: taipan-syntax
    path: specs/taipan-syntax.md
    depends: []
  - id: taipan-ir
    path: specs/taipan-ir.md
    depends: [taipan-syntax]
  - id: taipan-backend-cranelift
    path: specs/taipan-backend-cranelift.md
    depends: [taipan-ir]
  - id: taipan-cli-integration
    path: specs/taipan-cli-integration.md
    depends: [taipan-backend-cranelift]
---

<proposal>

# Change: cclab-taipan

## Summary

Implement Taipan high-performance compiler v0.1 with Cranelift backend and unified CLI integration.

## Why

The current Cclab ecosystem relies on several DSLs but lacks a general-purpose, high-performance compiler for executing complex, low-latency business logic. This forces a trade-off between the flexibility of interpreted languages like Python, which suffers from GIL contention, and the performance of native Rust, which requires redeployment for logic changes.\n\nTaipan bridges this gap by providing a performance-centric compiler pipeline that generates optimized machine code at runtime. By leveraging the pure-Rust Cranelift backend, Taipan ensures high execution speed with minimal integration overhead. This enables dynamic execution of performance-critical logic while maintaining the safety and speed characteristics of the platform.\n\nThis change implements v0.1 of Taipan, establishing the core pipeline from syntax analysis to code generation. It integrates Taipan into the unified CLI, making it a first-class tool within the ecosystem and setting the stage for advanced features like LLVM support and a full standard library in future iterations.

## What Changes

- Create new crates/cclab-taipan crate for the compiler core logic.
- Implement multi-stage analysis pipeline (Parser -> AST -> IR) using Aurora patterns.
- Develop Cranelift-based backend for high-performance machine code generation.
- Integrate 'taipan' command into cclab-cli via modular registration.
- Add minimal built-in functions including print() and basic arithmetic operators.
- Stub LLVM and WASM backends to support future extensibility.

## Impact

- **Scope**: minor
- **Affected Files**: ~20
- **New Files**: ~0
- Affected specs:
  - `cli-architecture` (no dependencies)
  - `aurora-codegen-system` (no dependencies)
  - `taipan-syntax` (no dependencies)
  - `taipan-ir` → depends on: `taipan-syntax`
  - `taipan-backend-cranelift` → depends on: `taipan-ir`
  - `taipan-cli-integration` → depends on: `taipan-backend-cranelift`
- Affected code: `crates/cclab-cli/src/main.rs`, `crates/cclab-cli/Cargo.toml`, `crates/cclab-taipan/`

</proposal>
