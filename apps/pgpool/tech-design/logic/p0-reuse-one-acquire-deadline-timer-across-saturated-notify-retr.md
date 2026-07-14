---
id: '1698'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-reused-acquire-deadline-timer-contract
entry: start
nodes:
  start: { kind: start, label: "Acquire loop initializes deadline and one pinned Sleep" }
  availability: { kind: decision, label: "Existing idle, replay, or permit path succeeds" }
  notify: { kind: process, label: "Enable fresh Notify future" }
  race: { kind: decision, label: "Select same Sleep or Notify" }
  retry: { kind: process, label: "Discard Notify only and recheck" }
  saturated: { kind: terminal, label: "Return PoolError::Saturated" }
  success: { kind: terminal, label: "Return existing acquisition outcome" }
edges:
  - { from: start, to: availability }
  - { from: availability, to: success, label: "yes" }
  - { from: availability, to: notify, label: "no" }
  - { from: notify, to: race }
  - { from: race, to: retry, label: "Notify" }
  - { from: retry, to: availability }
  - { from: race, to: saturated, label: "Sleep" }
---
flowchart LR
  init([one deadline Sleep]) --> available{existing path succeeds?}
  available -->|yes| grant([unchanged outcome])
  available -->|no| wait[enable Notify]
  wait --> race{same Sleep / Notify}
  race -->|Notify| available
  race -->|deadline| reject([existing saturation])
```

### Contract

- Each of the three acquisition functions owns one `Pin<Sleep>` for the fixed deadline. The sleep is created once, is never reset, and is polled only while the existing loop is saturated.
- A wait helper enables a newly-created `Notify` before selecting it against the caller-owned sleep. `Notify` completion preserves the current loop and creates a new notifier, not a new timer.
- The helper returns only `notified` or `deadline_elapsed`; callers retain all pre-existing resource checks and error construction. No helper receives a permit, stream, waiter record, or queue position.
- Cancellation drops the unconsumed timer and notifier without ownership side effects. No idle backend or semaphore permit is transferred by a timer event.

### Failure contract

Deadline completion returns exactly `self.saturated()`. Connect, bootstrap, liveness, reset, and dropped-lease failures continue through the current paths. A coalesced or spurious notification can cause an extra recheck but cannot defer the fixed deadline or admit above physical capacity.
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
