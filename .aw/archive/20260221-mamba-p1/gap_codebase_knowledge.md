---
change_id: mamba-p1
type: gap_codebase_knowledge
created_at: 2026-02-20T17:13:11.314589+00:00
updated_at: 2026-02-20T17:13:11.314589+00:00
---

# Gap Analysis: Codebase vs Knowledge

| Gap | Codebase Reference | Knowledge Reference | Severity |
|-----|--------------------|---------------------|----------|
| **Async Runtime Architectural Mismatch**: Specification requires true state-machine coroutines with suspension points and Orbit loop integration, but implementation is a one-shot executor that runs to completion on first step. | `crates/mamba/src/runtime/async_task.rs` (EventLoop, mb_await) | `main_spec:cclab-mamba/mamba-async-runtime.md#R1, R2` | High |
| **Missing Generator Lowering**: While runtime support for state-machine generators exists, the lowering pass in the compiler does not distinguish generator functions, treating them as regular functions with Yield as an extern call. | `crates/mamba/src/lower/hir_to_mir.rs` (HirExpr::Yield, lower_function) | `main_spec:cclab-mamba/mamba-codegen-logic.md#R2` | High |
| **Incomplete GC Root Tracking**: No automatic mechanism to track stack-allocated local variables as GC roots; relies on manual root management and global variable tracking. | `crates/mamba/src/runtime/gc.rs` (ROOTS, gc_add_root) | `main_spec:cclab-mamba/mamba-gc-runtime.md#R3` | Medium |
| **String-Based Type System**: `type()` returns strings instead of type objects, and `isinstance`/`issubclass` perform name-based checks rather than object-based identity/inheritance checks. | `crates/mamba/src/runtime/builtins.rs` (mb_type), `crates/mamba/src/runtime/class.rs` (mb_isinstance, mb_issubclass) | `main_spec:cclab-mamba/mamba-oop-model.md#R1`, `main_spec:cclab-mamba/mamba-stdlib-core.md` | Medium |
| **Non-Standard StopIteration Handling**: Iteration protocol relies on a thread-local 'exhausted' flag rather than a full exception-based `StopIteration` dispatch in the generated MIR. | `crates/mamba/src/runtime/iter.rs` (mb_next, mb_has_next) | `main_spec:cclab-mamba/mamba-iteration-protocol.md#R2` | Low |
| **Primitive Method Denial**: `mb_call_method` explicitly blocks method calls on primitive types (int, float, bool) which contradicts the 'everything is an object' Python model. | `crates/mamba/src/runtime/class.rs` (mb_call_method) | `main_spec:cclab-mamba/mamba-oop-model.md#R4` | Low |
