---
id: '1678'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-capacity-deadline-scheduler
entry: acquire
nodes:
  acquire: { kind: start, label: "Acquire checks reset-clean idle streams and physical permits before scheduler admission." }
  immediate: { kind: decision, label: "Can the caller atomically claim one available backend slot now?" }
  lease: { kind: process, label: "Return exactly one active lease, with existing liveness, connect, reset, and permit ownership behavior." }
  enqueue: { kind: process, label: "Append a FIFO waiter with an absolute deadline and oneshot result; scheduler updates its single earliest-deadline timer." }
  timer: { kind: decision, label: "Did a release grant this waiter, did the earliest scheduler deadline expire, or was the caller cancelled?" }
  grant: { kind: process, label: "One grant authorizes one recheck-and-claim attempt; it is not itself a backend permit." }
  expire: { kind: terminal, label: "Remove expired/cancelled waiter, preserve all physical permits, and return PoolError::Saturated for expiry." }
  release: { kind: process, label: "After an idle stream is visible or a physical permit is dropped, scheduler skips stale heads and resolves one oldest live waiter." }
  replay: { kind: terminal, label: "Replay cache publication broadcasts matching startup observers independently of capacity scheduling." }
edges:
  - { from: acquire, to: immediate }
  - { from: immediate, to: lease, label: "resource committed" }
  - { from: immediate, to: enqueue, label: "saturated" }
  - { from: enqueue, to: timer }
  - { from: timer, to: grant, label: "one-slot grant" }
  - { from: grant, to: immediate }
  - { from: timer, to: expire, label: "deadline or cancellation" }
  - { from: release, to: timer, label: "oldest live waiter" }
---
flowchart TD
    acquire([acquire]) --> immediate{idle stream or physical permit?}
    immediate -->|claim succeeds| lease[active backend lease]
    immediate -->|none| enqueue[enqueue FIFO waiter; update one earliest timer]
    enqueue --> timer{grant, expiry, or cancellation?}
    timer -->|one grant| grant[one authorized claim attempt]
    grant --> immediate
    timer -->|expiry/cancel| expire([remove waiter; saturated or cancelled])
    release[visible idle stream or dropped physical permit] --> handoff[skip stale heads; resolve oldest live waiter]
    handoff --> timer
    replay[startup replay cache publication] --> replay_broadcast([broadcast cache observers only])
```

### Contract invariants

- The scheduler never owns a backend stream or physical permit. An active lease or reset-clean idle tuple remains the sole owner of each physical permit.
- FIFO order applies to live saturated waiters. A release resolves exactly one oldest live waiter after capacity is visible; a grant only permits that waiter to attempt an atomic claim.
- Scheduler timer ownership is cardinality one: it is armed only for the earliest live deadline and is replaced only when that deadline changes, fires, or the queue empties.
- Every waiter preserves its original absolute deadline. Cancellation removes its scheduler record without converting to a backend grant; expiry returns the existing typed saturation error.
- Startup replay cache broadcast is not capacity handoff and cannot consume or create a physical backend permit.

### Error handling

Expired and cancelled waiters are removed before a handoff can target them. If a granted caller loses a race to a failed liveness probe, connect error, or another cancellation, it re-enters FIFO scheduling with its original deadline unless that deadline has elapsed. Any reset, close, dropped-lease, or failed-connect path invokes handoff only after its existing permit disposition completes, preventing phantom capacity or a lost wake.
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-capacity-deadline-scheduler
    impl_mode: hand-written
    reason: Own FIFO waiters, one earliest-deadline timer, exact cancellation, and one-slot capacity grants beside physical pool state.
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-capacity-deadline-scheduler
    impl_mode: hand-written
    reason: Verify FIFO grants, expiry, cancellation, and physical permit conservation.
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-capacity-deadline-scheduler
    impl_mode: hand-written
    reason: Verify capped transaction reuse, replay, reset isolation, and concurrent client behavior remain unchanged.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-capacity-deadline-scheduler-verification
requirements:
  deadline_and_cancellation:
    id: R3
    text: "Waiter expiry and cancellation preserve the original saturated error surface and cannot leak, duplicate, or steal a physical permit."
    kind: regression
    risk: high
    verify: pool::capacity_waiter_expiry_and_cancellation_preserve_permits
  fifo_handoff:
    id: R1
    text: "One available backend slot grants exactly one oldest live FIFO waiter, and successive releases preserve queue order without exceeding physical capacity."
    kind: regression
    risk: high
    verify: pool::fifo_capacity_handoff_admits_one_waiter_per_release
  performance_evidence:
    id: AC5
    text: "Meter is diagnostic only and the unchanged competitor benchmark is retained only after error-free unsampled release wins."
    kind: e2e
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh
  pool_contract:
    id: R4
    text: "Replay cache broadcast stays independent while transaction reuse, reset isolation, liveness, and session state remain correct under queued acquisition."
    kind: integration
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes --test trust_startup_replay
---
flowchart TD
    r1[R1 fifo handoff] --> pool_fifo_capacity_handoff_admits_one_waiter_per_release[pool::fifo_capacity_handoff_admits_one_waiter_per_release]
    r3[R3 deadline and cancellation] --> pool_capacity_waiter_expiry_and_cancellation_preserve_permits[pool::capacity_waiter_expiry_and_cancellation_preserve_permits]
    r4[R4 pool contract] --> cargo_test_p_pgpool_test_pool_test_pool_modes_test_trust_startup_replay[cargo test -p pgpool --test pool --test pool_modes --test trust_startup_replay]
    ac5[AC5 performance evidence] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh]
```
