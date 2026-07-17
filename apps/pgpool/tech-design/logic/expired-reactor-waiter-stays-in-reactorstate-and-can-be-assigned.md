---
id: '1879'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: expired-waiter-cannot-be-assigned
entry: expire_waiters
nodes:
  expire: { kind: process, label: "take expired deadline token" }
  close: { kind: process, label: "mark client Closing and queue 53300 error" }
  detach: { kind: process, label: "clear pending_first and remove_client" }
  flush: { kind: terminal, label: "close only after error output flushes" }
  guard: { kind: decision, label: "Assign target has non-Closing client and pending frame" }
  relay: { kind: terminal, label: "activate backend and relay first frame" }
edges:
  - { from: expire_waiters, to: expire }
  - { from: expire, to: close }
  - { from: close, to: detach }
  - { from: detach, to: flush }
  - { from: drive_assign, to: guard }
  - { from: guard, to: flush, label: invalid or Closing target }
  - { from: guard, to: relay, label: live pending first frame }
---
flowchart TD
  deadline["expired waiter deadline"] --> error["queue 53300 and Closing"]
  error --> detach["clear pending query; remove ReactorState waiter"]
  detach --> close["flush error then close"]
  backend["clean backend arrives"] --> guard{"Assign target live and eligible?"}
  guard -->|no| close
  guard -->|yes| relay["activate and relay query"]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/reactor/runtime.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: expire_waiters
  - path: apps/pgpool/src/pool/reactor/state.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: disconnected_waiters_do_not_consume_a_clean_backend
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: expired-waiter-cannot-be-assigned-verification
requirements:
  expired_waiter_is_removed:
    id: R1
    text: "Expiry removes the waiting identity from ReactorState and the stale pending first query cannot consume a later clean backend."
    kind: regression
    risk: high
    verify: cargo test -p pgpool expired_waiter_removal_prevents_late_assignment_and_accounting_underflow
  late_removal_is_accounting_safe:
    id: R2
    text: "A later close of the already-expired client is idempotent for waiting_count, while runtime Assign defensively ignores Closing clients."
    kind: regression
    risk: high
    verify: cargo test -p pgpool expired_waiter_removal_prevents_late_assignment_and_accounting_underflow
---
flowchart TD
    r1[R1 expired waiter is removed] --> cargo_test_p_pgpool_expired_waiter_removal_prevents_late_assignment_and_accounting_underflow[cargo test -p pgpool expired_waiter_removal_prevents_late_assignment_and_accounting_underflow]
    r2[R2 late removal is accounting safe] --> cargo_test_p_pgpool_expired_waiter_removal_prevents_late_assignment_and_accounting_underflow
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/reactor/runtime.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: expire_waiters
  - path: apps/pgpool/src/pool/reactor/state.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: disconnected_waiters_do_not_consume_a_clean_backend
```
