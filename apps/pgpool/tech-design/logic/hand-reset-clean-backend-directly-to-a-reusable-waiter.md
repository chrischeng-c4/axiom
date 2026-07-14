---
id: '1691'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-reset-clean-direct-handoff
entry: saturated_reusable_acquire
nodes:
  saturated_reusable_acquire:
    kind: start
    label: "Reusable acquire exhausted idle and physical capacity under its original deadline."
  enqueue:
    kind: process
    label: "Append one reusable-only oneshot sender; no timer driver or physical-capacity grant is created."
  receive:
    kind: decision
    label: "Receive exact reset-clean BackendLease before deadline."
  use_lease:
    kind: terminal
    label: "Lease arrives already outstanding and bypasses try_take_idle/liveness_check."
  ordinary_retry:
    kind: process
    label: "Existing Notify wake rechecks idle or semaphore without extending the deadline."
  saturation:
    kind: terminal
    label: "Timeout returns existing Saturated error; receiver closure is harmless."
  reset_done:
    kind: start
    label: "release ReturnToIdle completed DISCARD ALL and observed valid ReadyForQuery."
  pop_waiter:
    kind: decision
    label: "Pop the oldest reusable waiter whose receiver accepts a lease."
  send_lease:
    kind: process
    label: "Move stream with its original id/permit back to outstanding and send BackendLease."
  idle:
    kind: terminal
    label: "No receiver accepts: restore original idle tuple and generic Notify behavior."
edges:
  - from: saturated_reusable_acquire
    to: enqueue
  - from: enqueue
    to: receive
  - from: receive
    to: use_lease
    label: "handoff"
  - from: receive
    to: ordinary_retry
    label: "generic Notify"
  - from: ordinary_retry
    to: receive
  - from: receive
    to: saturation
    label: "deadline"
  - from: reset_done
    to: pop_waiter
  - from: pop_waiter
    to: send_lease
    label: "receiver live"
  - from: send_lease
    to: pop_waiter
    label: "receiver closed; lease returned"
  - from: pop_waiter
    to: idle
    label: "no live waiter"
  - from: send_lease
    to: use_lease
    label: "accepted"
---
flowchart LR
  wait([saturated reusable acquire]) --> q[one cancellable direct waiter]
  q --> r{reset-clean lease arrives?}
  r -->|yes| handed([return lease, no idle peek])
  r -->|notify| retry[ordinary resource recheck]
  retry --> r
  r -->|deadline| err([Saturated])

  reset([reset valid]) --> pop{live waiter?}
  pop -->|yes| send[move exact lease to receiver]
  send -->|closed| pop
  send -->|accepted| handed
  pop -->|no| idle([idle vector + Notify])
```

### Ownership rules

- The permit is never duplicated: before a successful direct send it is restored to `outstanding` for the same `BackendConnectionId`; an accepted lease owns the normal guard, while a failed send returns that same lease for another send or idle parking.
- A waiter is a delivery opportunity, not a reservation. Existing semaphore and generic Notify determine all fresh capacity and deadline behavior.
- A waiter queue is consulted only by `ReturnToIdle` after a successful reset; it cannot expose a stream after close/reset failure or before `DISCARD ALL` completes.
- `acquire_fresh` remains excluded. Reusable standard/replay transaction acquisitions may participate because the backend has already authenticated and reset.
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-reset-clean-direct-handoff
    impl_mode: hand-written
    reason: Register reusable-only waiters and transfer a reset-clean stream/permit directly without changing fresh admission or generic Notify fallback.
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-reset-clean-direct-handoff
    impl_mode: hand-written
    reason: Prove direct stream handoff skips idle re-acquisition and remains capacity/cancellation safe.
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-reset-clean-direct-handoff
    impl_mode: hand-written
    reason: Pin transaction reset isolation and backend-count stability while a reusable waiter receives a reset-clean connection.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-reset-clean-direct-handoff-verification
requirements:
  closed_waiter_recovery:
    id: R2
    text: "A cancelled or closed direct waiter cannot leak the stream or permit; a next live waiter receives it and pool accounting remains bounded."
    kind: regression
    risk: high
    verify: pool::cancelled_direct_handoff_waiter_passes_backend_to_next_waiter
  direct_stream_transfer:
    id: R1
    text: "A reusable acquisition already waiting at saturation receives the exact reset-clean backend directly rather than re-acquiring it from idle."
    kind: regression
    risk: high
    verify: pool::reset_clean_backend_hands_directly_to_waiting_reusable_acquire
  existing_paths:
    id: R4
    text: "Fresh-only startup, replay publication, saturation deadline, and session mode retain their existing contracts."
    kind: integration
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes --test trust_startup_replay
  peer_comparison:
    id: R5
    text: "The unchanged 64-client/16-backend/simple/30-second peer comparison has no errors; a first valid loss is immediately reverted as a no-go."
    kind: integration
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool
  reset_isolation:
    id: R3
    text: "Transaction state is reset before the next owner observes a direct handoff and repeated transaction reuse holds backend count stable."
    kind: integration
    risk: high
    verify: pool_modes::transaction_mode_direct_handoff_preserves_reset_isolation_and_capacity
---
flowchart TD
    r1[R1 direct stream transfer] --> pool_reset_clean_backend_hands_directly_to_waiting_reusable_acquire[pool::reset_clean_backend_hands_directly_to_waiting_reusable_acquire]
    r2[R2 closed waiter recovery] --> pool_cancelled_direct_handoff_waiter_passes_backend_to_next_waiter[pool::cancelled_direct_handoff_waiter_passes_backend_to_next_waiter]
    r3[R3 reset isolation] --> pool_modes_transaction_mode_direct_handoff_preserves_reset_isolation_and_capacity[pool_modes::transaction_mode_direct_handoff_preserves_reset_isolation_and_capacity]
    r4[R4 existing paths] --> cargo_test_p_pgpool_test_pool_test_pool_modes_test_trust_startup_replay[cargo test -p pgpool --test pool --test pool_modes --test trust_startup_replay]
    r5[R5 peer comparison] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_pgpool_bin_target_release_pgpool[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool]
```
