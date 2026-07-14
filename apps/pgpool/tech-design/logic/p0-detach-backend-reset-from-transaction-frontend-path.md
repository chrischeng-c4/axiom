---
id: '1639'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-detached-reset
entry: ready
nodes:
  ready: { kind: start, label: "Backend ReadyForQuery(Idle) validated and forwarded" }
  detach: { kind: process, label: "Remove lease permit into reset reservation and spawn reset task" }
  frontend: { kind: process, label: "Frontend task resumes next client activity" }
  reset: { kind: process, label: "Backend task sends DISCARD ALL and waits for its ReadyForQuery" }
  idle: { kind: process, label: "Park reset-clean stream and permit in idle set then notify" }
  dispose: { kind: process, label: "Drop failed/reset-cancelled stream and permit then notify" }
edges:
  - { from: ready, to: detach }
  - { from: detach, to: frontend }
  - { from: detach, to: reset }
  - { from: reset, to: idle, label: "reset ReadyForQuery" }
  - { from: reset, to: dispose, label: "error EOF timeout cancellation" }
---
flowchart LR
  ready([client sees ReadyForQuery Idle]) --> detach[move permit into reset reservation\nspawn backend-only reset]
  detach --> frontend[resume frontend task]
  detach --> reset[DISCARD ALL]
  reset -->|clean Ready| idle[park idle + notify]
  reset -->|fail/cancel| dispose[drop stream + permit + notify]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-detached-reset
    impl_mode: hand-written
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: pgpool-detached-reset
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-detached-reset
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-detached-reset
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-detached-reset-verification
requirements:
  capacity_isolation:
    id: R2
    text: "A resetting backend retains its permit, is never idle or reusable before successful reset, and reset error or task cancellation releases capacity exactly once."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes
  end_to_end:
    id: R3
    text: "Transaction mode preserves reset isolation, cap, and no client errors in the immutable benchmark."
    kind: e2e
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh
  frontend_progress:
    id: R1
    text: "Returning a transaction backend schedules its reset and returns before a deliberately delayed DISCARD ALL response, so the frontend path is not reset-bound."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool detached_reset_returns_before_backend_reset_completes
---
flowchart TD
    r1[R1 frontend progress] --> cargo_test_p_pgpool_test_pool_detached_reset_returns_before_backend_reset_completes[cargo test -p pgpool --test pool detached_reset_returns_before_backend_reset_completes]
    r2[R2 capacity isolation] --> cargo_test_p_pgpool_test_pool_test_pool_modes[cargo test -p pgpool --test pool --test pool_modes]
    r3[R3 end to end] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh]
```
