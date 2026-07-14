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

### Invariants

- `release(ReturnToIdle)` first removes the lease's permit from `outstanding`; after that point the ordinary lease guard is deliberately a no-op and the reset reservation is the sole owner of that permit.
- Scheduling is non-blocking for the transaction frontend. A reset task owns `(stream, permit)` until it either parks the stream in `idle` after its own `ReadyForQuery` or drops both on any failure.
- `idle` remains the only acquisition source. A resetting stream is never in that collection and therefore cannot be leased, reported idle, or receive client bytes before cleanup succeeds.
- The reset reservation's `Drop` drops its permit and wakes pool waiters. This covers reset read/write failure, timeout, task abort during runtime shutdown, and panic unwinding without a permit leak or a waiter stalled after capacity becomes available.
- On success the reservation transfers its permit exactly once into the idle tuple under `PoolState` mutex, then notifies waiters. On failure the reservation drops it exactly once and notifies.
- Existing client-visible ordering is unchanged: the valid backend response including `ReadyForQuery(Idle)` is forwarded before scheduling reset; malformed suffixes still close after their valid prefix. `DISCARD ALL` still completes before any later owner acquires the stream.

### Error handling

A spawn-owned reset converts every `reset_connection` error/EOF/timeout into stream disposal. The task never synthesizes a client response; its previous frontend has already received the completed transaction response. If runtime shutdown aborts the task, Rust drop order closes the stream and the reset reservation releases capacity plus notification. `Close` remains the existing synchronous disposal behavior for session teardown and terminal transaction legs.
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
