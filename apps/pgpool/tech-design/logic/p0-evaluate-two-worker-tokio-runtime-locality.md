---
id: '1708'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-two-worker-runtime-locality
entry: start
nodes:
  start: { kind: start, label: "pgpool process starts Tokio runtime" }
  two_workers: { kind: process, label: "Run existing multi-thread runtime with two workers" }
  serve: { kind: process, label: "Serve unchanged TCP and admin tasks" }
  benchmark: { kind: decision, label: "Normal 30-second comparison beats PgBouncer" }
  revert: { kind: terminal, label: "Revert worker count" }
  repeat: { kind: terminal, label: "Require independent clean repeats" }
edges:
  - { from: start, to: two_workers }
  - { from: two_workers, to: serve }
  - { from: serve, to: benchmark }
  - { from: benchmark, to: revert, label: "no" }
  - { from: benchmark, to: repeat, label: "yes" }
---
flowchart LR
  start([start]) --> workers[Tokio multi-thread: two workers]
  workers --> serve[unchanged pgpool service]
  serve --> compare{normal peer comparison wins?}
  compare -->|no| revert([revert])
  compare -->|yes| repeat([repeat proof])
```

### Contract

- The `pgpool` binary starts its existing `multi_thread` Tokio runtime with exactly two worker threads before dispatching its existing CLI subcommands.
- The `Send` task topology, asynchronous I/O and timer drivers, `tokio::spawn`, signal handling, TCP frontend, and admin plane remain unchanged.
- Transaction pooling remains unchanged: frontend admission, startup/replay, backend leasing, relay, reset, and capped physical capacity keep their existing implementations.
- No global queue, event interval, LIFO, blocking pool, dependency, pool primitive, timeout policy, queue/waiter behavior, socket operation, or wire frame ownership changes.

### Failure contract

- Any failed service behavior or first valid normal-baseline 30-second comparison that loses to PgBouncer reverts the worker-count change immediately.
- Meter output may diagnose worker contention but cannot decide candidate retention.
