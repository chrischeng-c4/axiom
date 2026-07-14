---
id: '1707'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-one-worker-event-polling
entry: start
nodes:
  start: { kind: start, label: "pgpool process starts" }
  builder: { kind: process, label: "Build multi-thread runtime with one worker, I/O/time, event interval one" }
  async_main: { kind: process, label: "Block on unchanged async command dispatch" }
  benchmark: { kind: decision, label: "Normal 30-second comparison beats PgBouncer" }
  revert: { kind: terminal, label: "Revert runtime bootstrap" }
  repeat: { kind: terminal, label: "Require independent clean repeats" }
edges:
  - { from: start, to: builder }
  - { from: builder, to: async_main }
  - { from: async_main, to: benchmark }
  - { from: benchmark, to: revert, label: "no" }
  - { from: benchmark, to: repeat, label: "yes" }
---
flowchart LR
  start([start]) --> builder[one worker + I/O/time + event interval 1]
  builder --> dispatch[unchanged async command dispatch]
  dispatch --> compare{normal peer comparison wins?}
  compare -->|no| revert([revert])
  compare -->|yes| repeat([repeat proof])
```

### Contract

- A synchronous `main` builds Tokio with `new_multi_thread`, `worker_threads(1)`, `enable_all`, and `event_interval(1)`, then blocks on the extracted `async_main`.
- `async_main` retains the existing CLI parsing and every command arm unchanged. I/O and time drivers remain enabled exactly as the macro supplied them.
- No global queue, LIFO, blocking-pool, dependency, pool, waiter, timeout, socket, wire, or relay setting changes.
- Transaction pooling remains unchanged: frontend admission, startup/replay, backend leasing, relay, reset, and capped physical capacity keep their existing implementations.

### Failure contract

- A bootstrap failure, behavior regression, or first valid normal-baseline 30-second comparison that loses to PgBouncer reverts the runtime bootstrap immediately.
- Meter output may diagnose event-polling effects but cannot decide candidate retention.
