---
id: improve-orbit-performance
type: proposal
version: 1
created_at: 2026-01-27T16:54:29.711799+00:00
updated_at: 2026-01-27T16:54:29.711799+00:00
author: mcp
status: proposed
iteration: 1
summary: "Optimize Orbit event loop by eliminating busy-waiting, reducing lock contention, and implementing high-performance timer/task management."
history:
  - timestamp: 2026-01-27T16:54:29.711799+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-27T16:54:50.080684+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 278.12
  - timestamp: 2026-01-27T16:56:20.436613+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 90.35
impact:
  scope: minor
  affected_files: 4
  new_files: 0
affected_specs:
  - id: orbit-core-optimization
    path: specs/orbit-core-optimization.md
    depends: []
  - id: orbit-internal-components
    path: specs/orbit-internal-components.md
    depends: []---

<proposal>

# Change: improve-orbit-performance

## Summary

Optimize Orbit event loop by eliminating busy-waiting, reducing lock contention, and implementing high-performance timer/task management.

## Why

The current cclab-orbit implementation suffers from critical performance bottlenecks: 1) Coroutine polling in `create_task` uses a naive 10ms sleep loop that holds the GIL, causing massive latency and GIL contention. 2) The TimerWheel uses a BTreeMap protected by a Mutex, which doesn't scale well for high volumes of timers. 3) The main task queue is protected by a Mutex, causing unnecessary contention between producers and the event loop. 4) The wakeup mechanism uses synchronous Condvars which are less efficient than Tokio-native notifications. These issues severely limit the throughput and responsiveness of Orbit as an asyncio replacement.

## What Changes

- Replace 10ms busy-waiting loop in coroutine polling with a proper Tokio-integrated Waker bridge
- Implement a hashed hierarchical timer wheel for O(1) timer operations, replacing the Mutex-wrapped BTreeMap
- Replace Mutex-wrapped task receiver with a lock-free or high-concurrency task queue structure
- Upgrade event loop wakeup mechanism from std::sync::Condvar to tokio::sync::Notify for sub-microsecond latency
- Optimize GIL acquisition batching in task processing to minimize overhead and maximize concurrent throughput

## Impact

- **Scope**: minor
- **Affected Files**: ~4
- **New Files**: ~0
- Affected specs:
  - `orbit-core-optimization` (no dependencies)
  - `orbit-internal-components` (no dependencies)
- Affected code: `crates/cclab-orbit/src/loop_impl.rs`, `crates/cclab-orbit/src/timer_wheel.rs`, `crates/cclab-orbit/src/task.rs`, `crates/cclab-orbit/src/handle.rs`

</proposal>
