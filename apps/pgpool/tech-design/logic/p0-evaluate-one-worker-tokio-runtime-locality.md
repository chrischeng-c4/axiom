---
id: '1702'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-one-worker-runtime-locality
entry: start
nodes:
  start: { kind: start, label: "pgpool process starts Tokio runtime" }
  one_worker: { kind: process, label: "Run existing multi-thread runtime with one worker" }
  serve: { kind: process, label: "Serve unchanged TCP and admin tasks" }
  benchmark: { kind: decision, label: "Clean 30-second comparison beats PgBouncer" }
  revert: { kind: terminal, label: "Revert runtime worker change" }
  repeat: { kind: terminal, label: "Require independent clean repeats" }
edges:
  - { from: start, to: one_worker }
  - { from: one_worker, to: serve }
  - { from: serve, to: benchmark }
  - { from: benchmark, to: revert, label: "no" }
  - { from: benchmark, to: repeat, label: "yes" }
---
flowchart LR
  start([start]) --> worker[Tokio multi-thread: one worker]
  worker --> serve[unchanged pgpool service]
  serve --> compare{clean peer comparison wins?}
  compare -->|no| revert([revert])
  compare -->|yes| repeat([repeat proof])
```

### Contract

- The `pgpool` binary starts its existing `multi_thread` Tokio runtime with exactly one worker thread before dispatching its existing CLI subcommands.
- The `Send` task topology, asynchronous I/O and timer drivers, `tokio::spawn`, signal handling, TCP frontend, and admin plane remain unchanged.
- Transaction pooling remains unchanged: frontend admission, startup/replay, backend leasing, relay, reset, and capped physical capacity keep their existing implementations.
- No pool primitive, timeout policy, queue/waiter behavior, socket operation, or wire frame ownership changes.

### Failure contract

- Any failed service behavior or first valid clean 30-second comparison that loses to PgBouncer reverts the runtime-worker change immediately.
- Meter output may diagnose worker lock contention but cannot decide candidate retention.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    section: pgpool-one-worker-runtime-locality
    impl_mode: hand-written
    reason: Constrain the existing Tokio multi-thread runtime to one worker without changing service, pool, wire, or CLI behavior.
```
