---
number: 748
title: "Test coverage: FFI — target 100% line coverage"
state: open
labels: [enhancement, P2, crate:mamba]
group: "ffi-coverage"
---

# #748 — Test coverage: FFI — target 100% line coverage

## Target
Line coverage: **100%** (critical safety boundary)

## Scope
- `src/ffi/` — foreign function interface, marshalling, memory bridge

## Rationale
FFI is a safety-critical boundary between Rust and external code. Every code path must be tested to prevent memory safety issues.

## Approach
1. Test all marshalling conversions (Rust ↔ Python types)
2. Test error handling for invalid FFI calls
3. Test memory bridge lifecycle (allocation, deallocation, safety checks)
4. Test all safety guards and bounds checks
