---
change: queue-test-coverage
group: queue-unit-test-coverage
date: 2026-04-04
---

# Requirements

Add unit tests (#[cfg(test)] modules) for all untested modules in cclab-queue crate:
- error.rs: Error type construction, Display impl, conversion traits (From impls)
- state.rs: Task state machine transitions, state enum variants, serialization
- ratelimit.rs: Rate limiter creation, token consumption, limit enforcement
- revocation.rs: Revocation store operations, task revocation checks
- metrics.rs: Metric recording, counter increments, histogram observations
- scheduler/delay.rs: Delayed task scheduling, delay computation
- scheduler/memory_backend.rs: In-memory scheduler backend CRUD operations
- scheduler/cloud_scheduler_backend.rs: Cloud Scheduler backend operations

Goal: Bring unit test coverage to parity with well-tested modules (broker, workflow). Each module should have tests covering happy path, edge cases, and error conditions.
