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
  acquire: { kind: start, label: "Transaction or replayed-startup acquisition checks reset-clean idle backends, then physical capacity." }
  immediate: { kind: decision, label: "Is an idle backend or fresh physical permit immediately available?" }
  lease: { kind: process, label: "Lease exactly one reset-clean backend or connect one fresh backend; no scheduler state remains." }
  enqueue: { kind: process, label: "Append one FIFO waiter with a oneshot sender and absolute acquire deadline; arm or update the single earliest-deadline timer." }
  wait: { kind: decision, label: "Did this waiter receive a one-slot grant, its cancellation drop, or the scheduler expiry result?" }
  retry: { kind: process, label: "Granted waiter rechecks idle and physical capacity, consumes exactly one slot, then re-arms only if a raced resource was withdrawn." }
  saturated: { kind: terminal, label: "Expired or cancelled waiter removes itself without a permit and returns PoolError::Saturated at its original deadline." }
  release: { kind: process, label: "Close, failed reset, dead idle disposal, or a reset-clean return makes at most one physical slot available." }
  handoff: { kind: process, label: "Scheduler discards stale head waiters, sends one grant to the oldest live waiter, and re-arms only the next earliest deadline." }
  replay: { kind: process, label: "Startup replay publication stores an exact safe reply then separately broadcasts cache observers; it does not enter capacity FIFO." }
edges:
  - { from: acquire, to: immediate }
  - { from: immediate, to: lease, label: "yes" }
  - { from: immediate, to: enqueue, label: "saturated" }
  - { from: enqueue, to: wait }
  - { from: wait, to: retry, label: "one-slot grant" }
  - { from: retry, to: lease, label: "resource committed" }
  - { from: retry, to: enqueue, label: "resource raced or removed" }
  - { from: wait, to: saturated, label: "deadline or cancellation" }
  - { from: release, to: handoff }
  - { from: handoff, to: wait, label: "oldest live waiter" }
---
flowchart TD
    acquire([acquire transaction backend]) --> immediate{idle stream or physical capacity?}
    immediate -->|yes| lease[lease one backend]
    immediate -->|no| enqueue[enqueue FIFO waiter with absolute deadline]
    enqueue --> wait{grant, expiry, or cancellation?}
    wait -->|one-slot grant| retry[recheck and commit one backend]
    retry -->|committed| lease
    retry -->|raced| enqueue
    wait -->|deadline/cancelled| saturated([PoolError::Saturated; no permit])
    release[close, dead idle, or reset-clean return] --> handoff[drop stale heads; grant oldest live waiter]
    handoff --> wait
    replay[startup replay publication] --> replay_broadcast[broadcast cache observers only]
```

### Invariants

- Physical capacity remains owned exactly once by an active lease or a reset-clean idle stream. Scheduler state contains only waiter identity, deadline, and notification sender; it never owns a backend stream or semaphore permit.
- Every saturated acquisition is ordered by FIFO insertion. A release removes cancelled or expired heads before granting one notification to the oldest live waiter, so one physical slot cannot admit more than one client.
- The scheduler holds at most one Tokio timer for its earliest live deadline. Adding, cancelling, granting, or expiring a waiter recomputes that earliest deadline only while holding scheduler state; no frontend owns an individual `Sleep`.
- A waiter that wakes does not assume ownership until it atomically commits an idle stream or physical permit. A stale wake, cancellation, liveness failure, or connect failure leaves no phantom permit and either advances the next waiter or returns the existing typed error.
- Startup replay is deliberately separate: exact safe reply publication wakes cache observers, while capacity release only drives FIFO backend admission.

### Error handling

When the earliest deadline fires, the scheduler removes every expired head in order, sends each a saturation result, and arms the next earliest live deadline. Dropping an acquire future marks or removes its waiter; a later release skips it without consuming a capacity handoff. Reset, liveness, connect, and dropped-lease failures preserve their existing stream disposal and physical-permit behavior, then invoke the same one-slot handoff only after capacity is actually visible.

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-capacity-deadline-scheduler
    impl_mode: hand-written
    reason: Replace per-waiter timeout and Notify capacity waits with pool-owned FIFO deadline scheduling and exact one-slot handoff.
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-capacity-deadline-scheduler
    impl_mode: hand-written
    reason: Prove FIFO capacity handoff, deadline expiry, cancellation cleanup, and permit conservation.
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-capacity-deadline-scheduler
    impl_mode: hand-written
    reason: Preserve transaction reuse, capped replay admission, reset isolation, and no session-state leak under queued acquisition.
```
