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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-current-thread-runtime-locality-verification
requirements:
  diagnostic_only:
    id: R3
    text: "Meter may diagnose scheduler contention but its instrumented TPS is diagnostic-only and cannot retain this candidate."
    kind: integration
    risk: medium
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --meter-bin target/debug/meter
  peer_proof:
    id: R2
    text: "A normal-baseline unchanged 64-client, 16-backend, simple-protocol transaction-pooling comparison completes for 30 seconds without errors and beats PgBouncer; the first valid loss reverts the candidate."
    kind: e2e
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool
  runtime_contract:
    id: R1
    text: "The binary builds with Tokio's current-thread runtime scheduler and preserves the transaction-pooling test surface."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes --test proxy --test trust_startup_replay --test wire_codec
---
flowchart TD
    r1[R1 runtime contract] --> cargo_test_p_pgpool_test_pool_test_pool_modes_test_proxy_test_trust_startup_replay_test_wire_codec[cargo test -p pgpool --test pool --test pool_modes --test proxy --test trust_startup_replay --test wire_codec]
    r2[R2 peer proof] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_pgpool_bin_target_release_pgpool[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool]
    r3[R3 diagnostic only] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_meter_bin_target_debug_meter[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --meter-bin target/debug/meter]
```
