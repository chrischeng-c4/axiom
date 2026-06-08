---
id: implementation
type: change_implementation
change_id: mamba-thread-safe
---

# Implementation

## Summary

Thread-safe Mamba runtime with safepoint-based STW GC and Tokio multi-threaded executor.

**R4 — Safepoint-based STW GC**: `gc_safepoint()` polls inserted at 6 deterministic points: `mb_coroutine_step`, `EventLoop::tick`, `mb_await` loop, `mb_run_until_complete` loop, `mb_call_method`/`mb_call_method1`, and `mb_next` (loop backedge). Collector thread exclusion via `IS_REGISTERED` thread_local prevents self-deadlock in `request_safepoint()`.

**R5 — Global shared async state**: COROUTINES, TASKS, WAKERS, TIMERS migrated from `thread_local!` to global `LazyLock<RwLock<HashMap>>`. All `.with(|x| x.borrow())` replaced with `.read().unwrap()` / `.write().unwrap()`.

**R6 — Tokio multi-threaded executor**: New `tokio_exec.rs` module with shared `OnceLock<tokio::runtime::Runtime>` (2 worker threads, auto GC thread registration). `mb_tokio_spawn()` and `mb_tokio_gather()` for parallel task execution. Registered in symbol table for JIT access.

**R7 — Atomic ID allocation**: `AtomicU64::fetch_add` for globally unique coroutine and task IDs.

**Tests**: 294 lib tests pass (5/5 runs stable). 6 integration tests in `thread_safety_tests.rs`: safepoint register/unregister, GC collect from registered thread (no deadlock), concurrent coroutine creation (ID uniqueness), multi-thread coroutine access, Tokio spawn from multiple threads, Tokio gather parallel execution.

## Changed Files

```
6 files changed

crates/mamba/src/runtime/async_rt.rs:
- Global LazyLock<RwLock<HashMap>> for COROUTINES and TASKS
- AtomicU64 for NEXT_CORO_ID/NEXT_TASK_ID
- Send+Sync impls for MbCoroutine and MbTask
- gc_safepoint() call in mb_coroutine_step

crates/mamba/src/runtime/async_task.rs:
- Global LazyLock<RwLock<HashMap>> for WAKERS and TIMERS
- gc_safepoint() calls in EventLoop::tick, mb_await loop, mb_run_until_complete loop
- Cancel task splits lock acquisition to avoid nested deadlock

crates/mamba/src/runtime/gc.rs:
- Safepoint protocol: SAFEPOINT_REQUESTED (AtomicBool), cooperative polling, Condvar sync
- IS_REGISTERED thread_local — collector thread excluded from wait count
- gc_register_thread() / gc_unregister_thread() for mutator lifecycle
- GC_TEST_LOCK serializes GC unit tests to fix global state pollution

crates/mamba/src/runtime/tokio_exec.rs (NEW):
- Shared Tokio multi-thread runtime via OnceLock (2 workers, auto GC registration)
- mb_tokio_spawn() — spawn coroutine as Tokio task
- mb_tokio_gather() — run multiple coroutines in parallel, block until all complete

crates/mamba/src/runtime/class.rs:
- gc_safepoint() in mb_call_method and mb_call_method1

crates/mamba/src/runtime/iter.rs:
- gc_safepoint() in mb_next (loop backedge)

crates/mamba/src/runtime/symbols.rs:
- Registered mb_tokio_spawn and mb_tokio_gather in symbol table

crates/mamba/src/runtime/mod.rs:
- Added pub mod tokio_exec

crates/mamba/Cargo.toml:
- Added tokio.workspace = true dependency

crates/mamba/tests/thread_safety_tests.rs (NEW):
- 6 integration tests for safepoint, concurrent coroutine, and Tokio execution
```

## Review: mamba-thread-safe-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-thread-safe

**Summary**: All spec requirements addressed. R1-R3 (atomic RC, RwLock collections, no-GIL) were completed in prior change. R4 safepoint polling now wired into 6 mutator hot paths with collector self-deadlock fix. R5 global async state migrated to LazyLock<RwLock<HashMap>>. R6 Tokio multi-threaded executor added (tokio_exec.rs with spawn/gather). R7 atomic ID counters. 294 lib tests pass stably (5/5 runs). 6 integration tests in thread_safety_tests.rs cover safepoints, concurrent coroutine creation, and Tokio parallel execution. MambaRuntime consolidation deferred as non-blocking refactor.

### Checklist

- [PASS] R1 Atomic Reference Counting (AtomicU32 in MbObject)
  - AtomicU32 with Acquire/Release ordering in rc.rs — completed in prior change
- [PASS] R2 Thread-Safe Core Collections (per-object RwLock)
  - List, Dict, Set, Instance fields wrapped in RwLock — completed in prior change
- [PASS] R3 No-GIL Execution Concurrency
  - GIL functions are thread_local no-ops; all operations use per-object locks — completed in prior change
- [PASS] R4 Safepoint-based STW GC wired into mutator execution
  - gc_safepoint() at mb_coroutine_step, EventLoop::tick, mb_await loop, mb_run_until_complete loop, mb_call_method/mb_call_method1, mb_next. IS_REGISTERED thread_local prevents collector self-deadlock.
- [PASS] R5 Global Shared Async State (off thread_local)
  - COROUTINES, TASKS, WAKERS, TIMERS all global LazyLock<RwLock<HashMap>>. State not consolidated into MambaRuntime struct but functionally equivalent.
- [PASS] R6 Tokio Executor Integration
  - New tokio_exec.rs: OnceLock<Runtime> with 2 workers, auto gc_register_thread. mb_tokio_spawn and mb_tokio_gather registered in symbol table. MbCoroutine/MbTask are Send+Sync.
- [PASS] R7 Atomic Global Counters
  - alloc_coro_id/alloc_task_id use AtomicU64::fetch_add(1, Relaxed)
- [PASS] Integration tests for safepoints and multi-thread async
  - 6 tests in thread_safety_tests.rs: safepoint register/unregister, GC collect from registered thread, concurrent coroutine creation, multi-thread coroutine access, Tokio spawn from multiple threads, Tokio gather parallel execution

### Issues

- **[LOW]** Async state split across module-level statics (COROUTINES, TASKS, WAKERS, TIMERS) rather than consolidated into a single MambaRuntime struct as spec suggests
  - *Recommendation*: Refactor into a single MambaRuntime struct in a future change — does not affect correctness or thread safety
