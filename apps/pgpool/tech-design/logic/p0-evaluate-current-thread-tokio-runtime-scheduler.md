---
id: '1706'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-current-thread-runtime-locality
entry: start
nodes:
  start: { kind: start, label: "pgpool process starts Tokio runtime" }
  current_thread: { kind: process, label: "Run existing service on current-thread scheduler" }
  serve: { kind: process, label: "Serve unchanged TCP and admin tasks" }
  benchmark: { kind: decision, label: "Clean 30-second comparison beats PgBouncer" }
  revert: { kind: terminal, label: "Revert scheduler flavor" }
  repeat: { kind: terminal, label: "Require independent clean repeats" }
edges:
  - { from: start, to: current_thread }
  - { from: current_thread, to: serve }
  - { from: serve, to: benchmark }
  - { from: benchmark, to: revert, label: "no" }
  - { from: benchmark, to: repeat, label: "yes" }
---
flowchart LR
  start([start]) --> runtime[Tokio current-thread]
  runtime --> serve[unchanged pgpool service]
  serve --> compare{clean peer comparison wins?}
  compare -->|no| revert([revert])
  compare -->|yes| repeat([repeat proof])
```

### Contract

- The `pgpool` binary starts its existing Tokio service on the `current_thread` scheduler before dispatching the same CLI subcommands.
- Tokio I/O and timer drivers, `tokio::spawn`, signal handling, TCP frontend, and admin plane keep their existing behavior; only the runtime scheduler flavor changes.
- Transaction pooling remains unchanged: frontend admission, startup/replay, backend leasing, relay, reset, and capped physical capacity keep their existing implementations.
- No pool primitive, timeout policy, queue/waiter behavior, socket operation, wire frame ownership, dependency version, or runtime tuning parameter changes.

### Failure contract

- Any failed service behavior or first valid normal-baseline 30-second comparison that loses to PgBouncer reverts the scheduler-flavor change immediately.
- Meter output may diagnose scheduler contention but cannot decide candidate retention.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    section: pgpool-current-thread-runtime-locality
    impl_mode: hand-written
    reason: Change only the Tokio scheduler flavor to current-thread while preserving all service and pool behavior.
```
