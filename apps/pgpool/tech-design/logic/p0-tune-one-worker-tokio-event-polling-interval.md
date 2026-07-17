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
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    section: pgpool-one-worker-event-polling
    impl_mode: hand-written
    reason: Replace only macro bootstrap with an equivalent explicit one-worker runtime builder that checks I/O and timers each scheduler tick.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-one-worker-event-polling-verification
requirements:
  diagnostic_only:
    id: R3
    text: "Meter may diagnose event-polling effects but its instrumented TPS is diagnostic-only and cannot retain this candidate."
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
    text: "The explicit Tokio builder preserves all pgpool transaction-pooling regression behavior while using one worker and event interval one."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes --test proxy --test trust_startup_replay --test wire_codec
---
flowchart TD
    r1[R1 runtime contract] --> cargo_test_p_pgpool_test_pool_test_pool_modes_test_proxy_test_trust_startup_replay_test_wire_codec[cargo test -p pgpool --test pool --test pool_modes --test proxy --test trust_startup_replay --test wire_codec]
    r2[R2 peer proof] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_pgpool_bin_target_release_pgpool[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool]
    r3[R3 diagnostic only] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_meter_bin_target_debug_meter[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --meter-bin target/debug/meter]
```
