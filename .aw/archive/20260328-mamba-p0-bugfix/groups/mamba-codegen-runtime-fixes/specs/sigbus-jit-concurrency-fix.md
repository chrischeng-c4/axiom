---
id: sigbus-jit-concurrency-fix
main_spec_ref: "crates/mamba/codegen/cranelift-jit"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, changes]
filled_sections: [overview, requirements, scenarios, changes]
create_complete: true
---

# Sigbus Jit Concurrency Fix

## Overview

`CraneliftJitBackend::new()` calls `cranelift_native::builder()` and `JITModule::new(jit_builder)` to create an isolated JIT compilation context per invocation. When `cargo test` runs conformance tests with multiple threads (default `--test-threads=N`), each test thread independently instantiates a `CraneliftJitBackend`, compiles MIR, calls `finalize_definitions()` (which invokes `mprotect` to mark pages executable), and transmutes the entry pointer to a callable `fn() -> i64`. On aarch64 (Apple Silicon), concurrent `JITModule` finalization causes SIGBUS — the underlying Cranelift memory allocator and `mprotect` calls are not safe for concurrent use across threads.

Single-threaded execution (`--test-threads=1`) passes because JIT init/finalize/execute never overlaps. The `gc.rs` module already uses `GC_TEST_LOCK: LazyLock<Mutex<()>>` to serialize GC-sensitive test paths — the same pattern applies here.

Fix: introduce a global `JIT_LOCK: LazyLock<Mutex<()>>` in `jit.rs` and acquire it around the entire JIT pipeline (`new()` → `codegen()` → execute entry) in the conformance runner's `run_and_capture()`. This serializes JIT operations across test threads without modifying the JIT backend itself.

Issue: #1114
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Global JIT serialization lock | P0 | A `JIT_LOCK: LazyLock<Mutex<()>>` exists in the JIT module and is acquirable from external callers (conformance runner). Concurrent JIT operations from separate test threads never overlap |
| R2 | Conformance runner acquires JIT lock | P0 | `run_and_capture()` acquires `JIT_LOCK` before `CraneliftJitBackend::new()` and holds it through entry point execution and `cleanup_all_generators()`. The guard is released after execution completes or on error |
| R3 | Multi-threaded conformance passes on aarch64 | P0 | `cargo test -p mamba --test conformance_tests` (default thread count) completes without SIGBUS on Apple Silicon. No `--test-threads=1` workaround needed |
| R4 | Single-threaded performance unaffected | P1 | Lock acquisition adds negligible overhead (<1ms) when only one thread is active. No contention in single-threaded mode |
| R5 | JIT backend API unchanged | P1 | `CraneliftJitBackend::new()`, `codegen()`, and the `CodegenOutput::Jit` path remain unchanged. The lock is external to the backend — callers opt into serialization |

### Constraints

- Fix is localized to: (1) exporting `JIT_LOCK` from `codegen/cranelift/jit.rs`, (2) acquiring it in `conformance/mod.rs` `run_and_capture()`
- The lock serializes the entire pipeline (init + compile + execute), not just init — `finalize_definitions()` and `mprotect` are also unsafe concurrently
- This is a short-term serialization fix; long-term per-thread JIT isolation is a separate enhancement
- Pattern follows `GC_TEST_LOCK: LazyLock<Mutex<()>>` in `gc.rs`
## Scenarios

### S1: Multi-threaded conformance tests pass without SIGBUS (R1, R2, R3)

**GIVEN** conformance test suite with 50+ `.py` fixtures
**WHEN** `cargo test -p mamba --test conformance_tests` runs with default thread count (>1)
**THEN** all tests complete without SIGBUS crash; previously-passing fixtures still pass

### S2: JIT lock serializes concurrent backend instantiation (R1, R2)

**GIVEN** two test threads T1 and T2 both calling `run_and_capture()` concurrently
**WHEN** T1 acquires `JIT_LOCK` and begins JIT compilation
**THEN** T2 blocks at `JIT_LOCK.lock()` until T1 finishes execution and releases the guard; T2 then proceeds without SIGBUS

### S3: Single-threaded execution remains functional (R4, R5)

**GIVEN** `cargo test -p mamba --test conformance_tests -- --test-threads=1`
**WHEN** each test runs sequentially
**THEN** lock is uncontended; all tests pass with no measurable overhead vs. pre-fix

### S4: JIT backend API unmodified (R5)

**GIVEN** external callers using `CraneliftJitBackend::new()` and `codegen()` (e.g., REPL, CLI driver)
**WHEN** they do not acquire `JIT_LOCK`
**THEN** behavior is unchanged — no lock required for single-threaded usage paths

### S5: Error during JIT compilation releases lock (R2)

**GIVEN** a test thread acquires `JIT_LOCK` and `CraneliftJitBackend::new()` or `codegen()` returns `Err`
**WHEN** the error propagates up
**THEN** the `MutexGuard` is dropped, releasing `JIT_LOCK`; subsequent threads are not deadlocked
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

```yaml
files:
  - path: crates/mamba/src/codegen/cranelift/jit.rs
    action: MODIFY
    desc: "Add `pub static JIT_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));` at module level. Import `std::sync::{LazyLock, Mutex}`. The lock is pub so the conformance runner can acquire it. No changes to CraneliftJitBackend methods."
  - path: crates/mamba/src/conformance/mod.rs
    action: MODIFY
    desc: "In `run_and_capture()`, acquire `let _jit_guard = crate::codegen::cranelift::jit::JIT_LOCK.lock().unwrap();` as the first statement, before `CraneliftJitBackend::new()`. The guard is held through backend init, codegen, entry point execution, and cleanup_all_generators(). The MutexGuard drops automatically at function exit (success or error), ensuring the lock is always released."
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
