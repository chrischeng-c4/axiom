---
change: mamba-thread-safe
group: mamba-thread-safe-refactor
date: 2026-03-07
---

# Requirements

1. **Global Runtime State:** Consolidate `COROUTINES`, `TASKS`, `WAKERS`, and `TIMERS` from `thread_local!` into a globally shared `MambaRuntime` struct, using synchronized collections (e.g., `DashMap` or `RwLock<HashMap>`) to ensure thread-safe access and task migration across threads.
2. **True Global Lock (GIL):** Replace the current mock thread-local `GIL_HELD` boolean with a real `Mutex`-protected Global Interpreter Lock if a GIL-based model is chosen, or fully transition to a GIL-free fine-grained locking model.
3. **Thread-Safe Garbage Collection:** Refactor the `GcState` and collection logic in `gc.rs` to support concurrent marking or a robust "Stop-the-World" mechanism that accurately scans roots across all mutator threads and their task-local/stack-local variables.
4. **Task Migration and Interop:** Enable `MbTask` and `MbCoroutine` to be safely sent and shared across OS threads, which is a prerequisite for integration with multi-threaded executors like Tokio.
5. **Atomic Global Counters:** Update all ID allocation logic (e.g., `alloc_coro_id`, `alloc_task_id`) to use `AtomicU64` counters instead of thread-local increments.
