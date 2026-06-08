---
id: cranelift-jit-memory-fix
main_spec_ref: "crates/mamba/codegen/cranelift-jit.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, changes, test-plan]
filled_sections: [overview, requirements, scenarios, changes, test-plan]
create_complete: true
---

# Cranelift Jit Memory Fix

## Overview

Replace cranelift-jit's `JITModule` with `ObjectModule` + custom `JitMemory` loader in `jit.rs`. Currently `CraneliftJitBackend` uses `JITModule::new()` / `finalize_definitions()` which calls `mprotect` internally — on macOS aarch64 this causes EXC_BAD_ACCESS (PAC failure / SIGBUS) under concurrent compilation because Cranelift's memory allocator is not thread-safe. The workaround `JIT_LOCK: LazyLock<Mutex<()>>` serializes all JIT operations, eliminating parallelism.

Fix: change `CraneliftJitBackend` to use `ObjectModule` (same as the AOT path in `aot.rs`) to produce in-memory Mach-O/ELF object bytes. A new `JitMemory` struct (in `jit_memory.rs`) parses the object with the `object` crate, loads `.text`/`.data` sections into `mmap(MAP_JIT)` pages on macOS (`mmap + mprotect` on Linux), resolves `mb_*` symbol relocations, and returns a callable function pointer. Each compilation allocates its own isolated memory region — no shared mutable state, no lock needed.

Key changes to `jit.rs`:
- Replace `JITModule` field with `ObjectModule`
- Replace `JITBuilder` init with `ObjectBuilder` init (same ISA setup)
- Replace `finalize_definitions()` + `get_finalized_function()` with `module.finish().emit()` + `JitMemory::load()`
- Remove `JIT_LOCK` export and all lock references
- `CodegenOutput::Jit { entry }` return type unchanged — pointer now points into `JitMemory` mmap region

Dependency changes: drop `cranelift-jit` from `Cargo.toml`; keep `cranelift-codegen`, `cranelift-frontend`, `cranelift-module`; add `cranelift-object` (already a dep for AOT).

Issue: #1114
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Replace JITModule with ObjectModule | P0 | `CraneliftJitBackend.module` field type changes from `JITModule` to `ObjectModule`. All function declaration, definition, and compilation calls use `ObjectModule` API. `cranelift-jit` crate is removed from `Cargo.toml` |
| R2 | Object emission replaces finalize_definitions | P0 | `module.finish().emit()` produces in-memory Mach-O (macOS) or ELF (Linux) bytes. No `finalize_definitions()` or `mprotect` calls from cranelift-jit remain in the codebase |
| R3 | JitMemory loads and executes emitted objects | P0 | `JitMemory::load(object_bytes, symbols)` parses `.text` section, allocates executable pages via `mmap(MAP_JIT)` on macOS / `mmap + mprotect` on Linux, resolves relocations against `mb_*` symbol addresses, and returns a function pointer to the entry |
| R4 | Remove JIT_LOCK serialization | P0 | `JIT_LOCK: LazyLock<Mutex<()>>` is removed from `jit.rs`. Lock acquisition in `conformance/mod.rs` `run_and_capture()` is removed. Each compilation is independently thread-safe via isolated mmap regions |
| R5 | Concurrent multi-threaded conformance passes | P0 | `cargo test -p mamba --test conformance_tests` with default thread count completes without SIGBUS on aarch64 macOS. No `--test-threads=1` workaround needed |
| R6 | CodegenOutput::Jit return type unchanged | P1 | `compile_module()` still returns `CodegenOutput::Jit { entry }` with a `fn() -> i64` pointer. Callers (conformance runner, REPL driver) require no changes |
| R7 | Runtime symbol resolution preserved | P1 | All `mb_*` runtime functions are registered as external symbols during ObjectModule setup. `JitMemory` resolves them at load time. Same symbol set as current `JITBuilder::symbol()` calls |
| R8 | REPL incremental compilation preserved | P1 | REPL sessions that define functions in iteration N and call them in iteration N+1 continue to work. Each REPL iteration produces a fresh `ObjectModule` + `JitMemory` region |
| R9 | Memory cleanup on drop | P2 | `JitMemory` implements `Drop` to `munmap` allocated pages. No memory leaks after backend disposal |

### Constraints

- `cranelift-object` is already a dependency (used by `aot.rs`); no new external crate beyond `object` (for parsing) and `libc` (for mmap)
- `jit_memory.rs` is a new file alongside `jit.rs` — platform-specific code uses `#[cfg(target_os = "macos")]` / `#[cfg(not(target_os = "macos"))]`
- The `object` crate version must match the one transitively used by `cranelift-object` to avoid duplicate types
- Relocation handling: cranelift-module resolves intra-module relocations at compile time; only external symbol fixups needed at load time
## Scenarios

### S1: Multi-threaded conformance tests pass without SIGBUS (R4, R5)

**GIVEN** conformance test suite with 50+ `.py` fixtures
**WHEN** `cargo test -p mamba --test conformance_tests` runs with default thread count (>1) on aarch64 macOS
**THEN** all tests complete without SIGBUS or EXC_BAD_ACCESS; each test thread compiles and executes independently via isolated `JitMemory` regions

### S2: ObjectModule produces valid executable code (R1, R2, R3)

**GIVEN** a MIR module with one entry function calling `mb_print`
**WHEN** `CraneliftJitBackend` compiles it via `ObjectModule` → `finish().emit()` → `JitMemory::load()`
**THEN** the returned function pointer is callable, executes the compiled code correctly, and `mb_print` resolves to the correct runtime address

### S3: JIT_LOCK removed — no serialization bottleneck (R4)

**GIVEN** the updated `jit.rs` without `JIT_LOCK`
**WHEN** two test threads T1 and T2 both call `run_and_capture()` concurrently
**THEN** both compile and execute in parallel (no blocking) because each has its own `ObjectModule` + `JitMemory` instance with isolated mmap regions

### S4: REPL incremental compilation works (R6, R8)

**GIVEN** a REPL session using `CraneliftJitBackend`
**WHEN** user defines function `foo` in iteration 1 and calls `foo()` in iteration 2
**THEN** iteration 2 compiles a fresh `ObjectModule` that resolves `foo` as an external symbol, loads via `JitMemory`, and executes correctly

### S5: Runtime symbol resolution for all mb_* functions (R7)

**GIVEN** a compiled module that calls `mb_alloc`, `mb_get_attr`, `mb_dispatch_binop`, `mb_print`
**WHEN** `JitMemory::load()` processes relocations in the emitted object
**THEN** all `mb_*` external symbol references are resolved to correct runtime addresses; calling any of them from JIT-compiled code produces correct results

### S6: Memory cleanup on backend drop (R9)

**GIVEN** a `CraneliftJitBackend` that has compiled and loaded code into `JitMemory`
**WHEN** the backend is dropped
**THEN** `JitMemory::drop()` calls `munmap` on all allocated pages; no memory leak after drop

### S7: CodegenOutput::Jit API unchanged for callers (R6)

**GIVEN** external callers (conformance runner, REPL, CLI driver) using `compile_module()` result
**WHEN** they pattern-match `CodegenOutput::Jit { entry }`
**THEN** behavior is identical — `entry` is a valid `fn() -> i64` pointer, caller code requires no changes
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

| Test | Type | Covers | Description |
|------|------|--------|-------------|
| `test_objectmodule_compile_and_load` | unit | S2, R1, R2, R3 | Compile a minimal MIR module (single function returning constant) via ObjectModule path, load with JitMemory, call entry pointer, verify correct return value |
| `test_runtime_symbol_resolution` | unit | S5, R7 | Compile a function that calls `mb_print` and `mb_add`, load via JitMemory, execute, verify runtime functions are called with correct arguments |
| `test_concurrent_jit_no_sigbus` | integration | S1, S3, R4, R5 | Spawn 4+ threads each compiling and executing an independent MIR module simultaneously. Assert all complete without SIGBUS/EXC_BAD_ACCESS on aarch64 macOS |
| `test_conformance_multi_threaded` | integration | S1, R5 | Run `cargo test -p mamba --test conformance_tests` with default thread count. All previously-passing fixtures still pass. No `--test-threads=1` workaround |
| `test_jit_memory_cleanup_on_drop` | unit | S6, R9 | Create JitMemory, drop it, verify munmap was called (or verify no address-space leak via `/proc/self/maps` on Linux / `vmmap` proxy on macOS) |
| `test_repl_incremental_objectmodule` | integration | S4, R8 | Simulate two REPL iterations: iter 1 defines `foo`, iter 2 calls `foo()`. Both use ObjectModule → JitMemory path. Verify correct execution |
| `test_codegen_output_type_unchanged` | unit | S7, R6 | Call `compile_module()`, pattern-match result as `CodegenOutput::Jit { entry }`, verify entry is a valid non-null function pointer |

### Regression watchlist

- All existing conformance test fixtures must pass unchanged
- REPL sessions (interactive compilation) must work identically
- AOT path (`aot.rs`) must be unaffected — it already uses ObjectModule independently
- NaN-boxing and recursive function tests from previous change (`cranelift-jit` spec R6/R7) must still pass
- `JIT_LOCK` tests are removed (lock no longer exists) — not a regression, intentional removal
## Changes

```yaml
files:
  - path: crates/mamba/src/codegen/cranelift/jit.rs
    action: MODIFY
    desc: |
      Replace JITModule with ObjectModule throughout.

      Remove:
      - `use cranelift_jit::{JITBuilder, JITModule};`
      - `pub static JIT_LOCK: LazyLock<Mutex<()>>` and its `use std::sync::{LazyLock, Mutex}` import
      - All `finalize_definitions()` calls
      - `get_finalized_function()` calls
      - JITBuilder symbol registration loop

      Add:
      - `use cranelift_object::{ObjectBuilder, ObjectModule};`
      - `use crate::codegen::cranelift::jit_memory::JitMemory;`

      Modify:
      - `CraneliftJitBackend.module` field: `Option<JITModule>` → `Option<ObjectModule>`
      - `new()`: replace JITBuilder/JITModule init with ObjectBuilder/ObjectModule init (same ISA, same flag_builder). Collect mb_* symbol addresses into a HashMap for JitMemory
      - `module()`: return `&mut ObjectModule` instead of `&mut JITModule`
      - `compile_module()` final section: replace `finalize_definitions()` + `get_finalized_function()` with `module.take().finish().emit()` → `JitMemory::load(bytes, symbols)` → return entry pointer
      - Store `JitMemory` handle in backend to keep mmap alive until drop

      Remove tests:
      - All `JIT_LOCK` tests (test_jit_lock_exists, test_jit_lock_serializes_concurrent_access, etc.) — the lock no longer exists

  - path: crates/mamba/src/codegen/cranelift/jit_memory.rs
    action: CREATE
    desc: |
      New file — see companion spec `jit-memory` for full specification.
      This change spec only covers jit.rs modifications; jit_memory.rs is specified in the `jit-memory` spec.

  - path: crates/mamba/src/codegen/cranelift/mod.rs
    action: MODIFY
    desc: |
      Add `pub mod jit_memory;` to module declarations.

  - path: crates/mamba/src/conformance/mod.rs
    action: MODIFY
    desc: |
      Remove JIT_LOCK acquisition from `run_and_capture()`:
      - Delete `let _jit_guard = crate::codegen::cranelift::jit::JIT_LOCK.lock().unwrap();`
      - No replacement needed — each test thread now has isolated JitMemory

  - path: crates/mamba/Cargo.toml
    action: MODIFY
    desc: |
      Remove:
      - `cranelift-jit = "0.116"` dependency

      Keep:
      - `cranelift-codegen`, `cranelift-frontend`, `cranelift-module` (unchanged)
      - `cranelift-object` (already present for AOT path)

      Add (if not already present):
      - `object` crate (for parsing emitted Mach-O/ELF to find .text section) — version must match cranelift-object's transitive dep
      - `libc` (for mmap/munmap/mprotect FFI) — likely already a transitive dep
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
