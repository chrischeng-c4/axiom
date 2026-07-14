---
id: '1698'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-reused-acquire-deadline-timer
entry: deadline
nodes:
  deadline: { kind: start, label: "Fix acquire deadline and pin one Sleep" }
  check: { kind: decision, label: "Idle reuse, replay, or physical permit available" }
  acquire: { kind: terminal, label: "Return existing acquisition result" }
  enable: { kind: process, label: "Create and enable a fresh Notify future" }
  wait: { kind: decision, label: "Select shared deadline Sleep against Notify" }
  retry: { kind: process, label: "Wake and recheck existing acquisition loop" }
  timeout: { kind: terminal, label: "Return existing saturated error" }
edges:
  - { from: deadline, to: check }
  - { from: check, to: acquire, label: "available" }
  - { from: check, to: enable, label: "saturated" }
  - { from: enable, to: wait }
  - { from: wait, to: retry, label: "Notify" }
  - { from: retry, to: check }
  - { from: wait, to: timeout, label: "deadline" }
---
flowchart LR
  start([fix deadline + pin one Sleep]) --> check{capacity or replay available?}
  check -->|yes| granted([existing result])
  check -->|no| notify[enable fresh Notify future]
  notify --> race{Notify or same Sleep?}
  race -->|Notify| check
  race -->|deadline| saturated([existing PoolError::Saturated])
```

### Invariants

- Each invocation fixes one deadline and owns exactly one pinned `Sleep`; Notify wakeups never reset or replace it.
- A fresh enabled `Notify` future retains the current no-missed-wake protocol. A wake only re-enters the existing idle/replay/physical-capacity checks; it grants no slot by itself.
- The deadline race replaces only the current `timeout(remaining, notified)` wrapper. All capacity permits, Notify cardinality, timeout error shape, reset, liveness, and client-visible behavior remain unchanged.
- The timer exists only while saturated acquisition waits; it is not added to the backend relay leg.

### Error handling

If the shared deadline wins, return the existing saturated error. If Notify wins, drop only that one-shot notifier and retry with the same pending Sleep. Connect, bootstrap, reset, liveness, cancellation, and dropped-lease errors preserve their existing paths and permit cleanup.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-reused-acquire-deadline-timer
    impl_mode: hand-written
    reason: Reuse one pinned deadline Sleep in each existing saturated acquisition loop without changing pool ownership or wake policy.
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-reused-acquire-deadline-timer
    impl_mode: hand-written
    reason: Verify repeated wakeups retain the acquisition deadline and a released backend remains safely reusable.
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-reused-acquire-deadline-timer
    impl_mode: hand-written
    reason: Retain real transaction-mode contention, reset isolation, and capped startup replay coverage.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-reused-acquire-deadline-timer-verification
requirements:
  capacity_safety:
    id: R2
    text: "A returned backend remains reusable by a waiting transaction while physical capacity, reset isolation, and cancellation cleanup remain exact."
    kind: integration
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes --test trust_startup_replay
  meter_diagnosis:
    id: R3
    text: "Meter attributes less scheduler work to retry Sleep reset/drop after timer reuse; sampled TPS is not peer proof."
    kind: integration
    risk: medium
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --meter-bin target/debug/meter
  peer_gate:
    id: R4
    text: "Only three clean unchanged unsampled release comparisons matching or exceeding PgBouncer retain the candidate; the first valid loss fully reverts it."
    kind: e2e
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool
  reused_deadline:
    id: R1
    text: "Repeated saturated Notify wakeups recheck pool state against one fixed acquisition deadline and do not extend the existing timeout."
    kind: regression
    risk: high
    verify: pool::saturated_waiter_keeps_deadline_across_spurious_wakeups
---
flowchart TD
    r1[R1 reused deadline] --> pool_saturated_waiter_keeps_deadline_across_spurious_wakeups[pool::saturated_waiter_keeps_deadline_across_spurious_wakeups]
    r2[R2 capacity safety] --> cargo_test_p_pgpool_test_pool_test_pool_modes_test_trust_startup_replay[cargo test -p pgpool --test pool --test pool_modes --test trust_startup_replay]
    r3[R3 meter diagnosis] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_meter_bin_target_debug_meter[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --meter-bin target/debug/meter]
    r4[R4 peer gate] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_pgpool_bin_target_release_pgpool[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool]
```
