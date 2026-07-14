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
entry: reusable_acquire_saturated
nodes:
  reusable_acquire_saturated:
    kind: start
    label: "A reusable acquisition finds neither idle stream nor physical permit before its existing deadline."
  register_waiter:
    kind: process
    label: "Register one cancellable reusable-only oneshot waiter; retain existing Notify as the generic capacity fallback."
  wait_result:
    kind: decision
    label: "Did the waiter receive a reset-clean BackendLease before its original deadline?"
  handoff_lease:
    kind: terminal
    label: "Return the exact handed-off stream with permit already restored to outstanding; do not run idle liveness."
  fallback_recheck:
    kind: process
    label: "On generic Notify or spurious wake, recheck existing idle/permit paths without extending the deadline."
  timeout:
    kind: terminal
    label: "Return existing Saturated error and leave a closed receiver for release to skip."
  ready_idle:
    kind: start
    label: "A transaction backend completed valid DISCARD ALL ReadyForQuery and owns its stream plus permit."
  next_live_waiter:
    kind: decision
    label: "An oldest live reusable waiter is registered."
  transfer:
    kind: process
    label: "Restore outstanding ownership and send the exact BackendLease directly to that waiter."
  skip_closed:
    kind: process
    label: "A cancelled receiver returns its unsent lease; try the next waiter without dropping stream or permit."
  park_idle:
    kind: terminal
    label: "No live reusable waiter: park the reset-clean stream in idle and retain existing Notify behavior."
edges:
  - from: reusable_acquire_saturated
    to: register_waiter
  - from: register_waiter
    to: wait_result
  - from: wait_result
    to: handoff_lease
    label: "direct lease"
  - from: wait_result
    to: fallback_recheck
    label: "Notify/spurious wake"
  - from: wait_result
    to: timeout
    label: "deadline"
  - from: fallback_recheck
    to: wait_result
    label: "still saturated"
  - from: ready_idle
    to: next_live_waiter
  - from: next_live_waiter
    to: transfer
    label: "yes"
  - from: next_live_waiter
    to: park_idle
    label: "no"
  - from: transfer
    to: skip_closed
    label: "receiver closed"
  - from: skip_closed
    to: next_live_waiter
  - from: transfer
    to: handoff_lease
    label: "receiver accepts"
---
flowchart TD
  wait([reusable acquire saturated]) --> register[register cancellable direct waiter]
  register --> result{direct lease before deadline?}
  result -->|yes| lease([return reset-clean lease; no peek])
  result -->|notify/spurious| retry[existing idle/permit recheck]
  retry --> result
  result -->|deadline| saturated([existing Saturated error])

  reset([DISCARD ALL valid ReadyForQuery]) --> waiter{live reusable waiter?}
  waiter -->|yes| send[send exact stream + permit as BackendLease]
  send -->|closed| waiter
  send -->|accepted| lease
  waiter -->|no| idle([park idle; existing Notify])
```

### Contract invariants

- Only successful reset completion creates a direct handoff. EOF, reset error, or timeout still closes the socket and releases physical capacity through the existing path.
- The handed-off `BackendLease` owns the original stream and the same permit; its `CapacityGuard` observes it as outstanding before the receiver can use or drop it.
- A direct waiter is reusable-only. `acquire_fresh` and startup replay publication never receive an authenticated idle stream through this queue.
- Receiver cancellation cannot leak capacity: a failed oneshot send returns the lease to release, which tries the next live waiter before parking it idle. Timeout does not change the caller's deadline or reset the generic Notify path.
- Direct handoff has no timer driver, global scheduler, or new connection decision. It removes only the reset-clean idle-vector and liveness-probe round trip.
