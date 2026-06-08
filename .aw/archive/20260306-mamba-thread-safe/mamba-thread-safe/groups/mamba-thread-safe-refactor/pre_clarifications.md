---
change: mamba-thread-safe
group: mamba-thread-safe-refactor
date: 2026-03-07
status: answered
---

# Pre-Clarifications

### Q1: GIL Architecture
- **Answer**: No-GIL design. Continue the fine-grained locking path already started in rc.rs (per-object RwLock + AtomicU32 refcount). No global interpreter lock.

### Q2: GC Synchronization
- **Answer**: Safepoint-based stop-the-world. The interpreter checks a global flag periodically (function entry, loop backedge). Cooperative and portable — no OS thread suspension.

### Q3: Task Scheduling
- **Answer**: Tokio integration. Migrate thread_local COROUTINES/TASKS/WAKERS/TIMERS to global shared state (DashMap or similar), and bridge Mamba tasks into Tokio's multi-threaded executor for true parallel async I/O.

### Q4: Contention
- **Answer**: No specific hotspots to focus on. General thread-safety pass across the runtime, optimize later based on profiling.

