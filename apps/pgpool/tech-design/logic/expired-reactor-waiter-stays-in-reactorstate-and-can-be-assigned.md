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
  expired: { kind: process, label: "deadline expires while client is Waiting" }
  reject: { kind: process, label: "queue saturation ErrorResponse and mark Closing" }
  remove: { kind: process, label: "remove client state and clear pending_first" }
  assign_guard: { kind: decision, label: "Assign target is still eligible" }
  close: { kind: terminal, label: "flush error then close socket" }
  assign: { kind: terminal, label: "assign clean backend and relay pending query" }
edges:
  - { from: expire_waiters, to: expired }
  - { from: expired, to: reject }
  - { from: reject, to: remove }
  - { from: remove, to: close }
  - { from: assign_or_park, to: assign_guard }
  - { from: assign_guard, to: close, label: Closing or absent client }
  - { from: assign_guard, to: assign, label: Active eligible waiter }
---
flowchart TD
  expired["waiter deadline expires"] --> reject["queue saturation error and Closing"]
  reject --> remove["remove state + clear pending_first"]
  remove --> flush["flush error then close"]
  free["backend becomes clean"] --> guard{"Assign client still eligible?"}
  guard -->|Closing or missing| flush
  guard -->|eligible| assign["assign backend and relay query"]
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
