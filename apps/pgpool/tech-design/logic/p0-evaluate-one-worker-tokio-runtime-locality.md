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
    reason: Configure exactly one worker on the existing Tokio multi-thread runtime while leaving all service and pool semantics intact.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-one-worker-runtime-locality-verification
requirements:
  diagnostic_only:
    id: R3
    text: "Meter may confirm reduced runtime parking contention but its instrumented TPS is diagnostic-only and cannot retain this candidate."
    kind: integration
    risk: medium
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --meter-bin target/debug/meter
  peer_proof:
    id: R2
    text: "A clean unchanged 64-client, 16-backend, simple-protocol transaction-pooling comparison completes for 30 seconds without errors and beats PgBouncer; the first valid loss reverts the candidate."
    kind: e2e
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool
  runtime_contract:
    id: R1
    text: "The binary builds with the existing Tokio multi-thread runtime constrained to exactly one worker and preserves the transaction-pooling test surface."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes --test proxy --test trust_startup_replay --test wire_codec
---
flowchart TD
    r1[R1 runtime contract] --> cargo_test_p_pgpool_test_pool_test_pool_modes_test_proxy_test_trust_startup_replay_test_wire_codec[cargo test -p pgpool --test pool --test pool_modes --test proxy --test trust_startup_replay --test wire_codec]
    r2[R2 peer proof] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_pgpool_bin_target_release_pgpool[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool]
    r3[R3 diagnostic only] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_meter_bin_target_debug_meter[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --meter-bin target/debug/meter]
```
