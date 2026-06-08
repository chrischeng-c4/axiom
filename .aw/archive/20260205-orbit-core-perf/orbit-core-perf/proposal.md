---
id: orbit-core-perf
type: proposal
version: 1
created_at: 2026-02-05T04:27:21.120512+00:00
updated_at: 2026-02-05T04:27:21.120512+00:00
author: mcp
status: proposed
iteration: 1
summary: "Implement 5 core performance optimizations for cclab-orbit event loop"
history:
  - timestamp: 2026-02-05T04:27:21.120512+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 12
  new_files: 2
affected_specs:
  - id: gil-waker-polling
    path: specs/gil-waker-polling.md
    depends: []
  - id: mpsc-task-queue
    path: specs/mpsc-task-queue.md
    depends: [gil-waker-polling]
  - id: hashed-timer-wheel
    path: specs/hashed-timer-wheel.md
    depends: [mpsc-task-queue]
  - id: adaptive-gil-batching
    path: specs/adaptive-gil-batching.md
    depends: [hashed-timer-wheel]
  - id: io-uring-backend
    path: specs/io-uring-backend.md
    depends: [adaptive-gil-batching]
---

<proposal>

# Change: orbit-core-perf

## Summary

Implement 5 core performance optimizations for cclab-orbit event loop

## Why

The orbit event loop currently has several performance bottlenecks that limit its effectiveness as a Python async runtime:

1. **GIL Contention (#58)**: The current `create_task` uses a 10ms sleep loop while polling coroutines, holding the GIL during sleep and blocking other Python threads. This causes high latency (10ms+) for short-lived coroutines and poor CPU utilization.

2. **Timer Inefficiency (#59)**: The timer wheel uses `Mutex<BTreeMap>` with O(log n) operations. A hashed hierarchical timer wheel implementation exists in `timer_wheel_hashed.rs` but isn't integrated.

3. **Task Queue Contention (#60)**: The task queue uses `Mutex<UnboundedReceiver>`, causing lock contention under high load.

4. **Fixed Batching (#61)**: Minimal GIL batching with fixed sizes doesn't adapt to varying workloads, leading to suboptimal latency/throughput tradeoffs.

5. **I/O Overhead (#103)**: On Linux, epoll-based I/O has higher syscall overhead compared to io_uring, which offers zero-copy I/O and reduced context switches.

These optimizations are sequential dependencies: GIL fix enables efficient polling → MPSC queue enables fast task dispatch → timer wheel improves scheduling → batching optimizes GIL patterns → io_uring leverages all previous work for maximum I/O performance.

## What Changes

- Replace 10ms sleep loop with waker-driven coroutine polling (#58)
- Implement lock-free MPSC task queue using crossbeam channels (#60)
- Integrate hashed hierarchical timer wheel for O(1) timer operations (#59)
- Add adaptive GIL batching based on queue depth (#61)
- Implement io_uring backend for Linux with epoll fallback (#103)

## Impact

- **Scope**: minor
- **Affected Files**: ~12
- **New Files**: ~2
- Affected specs:
  - `gil-waker-polling` (no dependencies)
  - `mpsc-task-queue` → depends on: `gil-waker-polling`
  - `hashed-timer-wheel` → depends on: `mpsc-task-queue`
  - `adaptive-gil-batching` → depends on: `hashed-timer-wheel`
  - `io-uring-backend` → depends on: `adaptive-gil-batching`
- Affected code: `crates/cclab-orbit/src/executor.rs`, `crates/cclab-orbit/src/task.rs`, `crates/cclab-orbit/src/waker.rs`, `crates/cclab-orbit/src/timer_wheel.rs`, `crates/cclab-orbit/src/timer_wheel_hashed.rs`, `crates/cclab-orbit/src/loop_impl.rs`, `crates/cclab-orbit/src/lib.rs`, `crates/cclab-orbit/src/network.rs`, `crates/cclab-orbit/src/file_io.rs`, `crates/cclab-orbit/Cargo.toml`

</proposal>
