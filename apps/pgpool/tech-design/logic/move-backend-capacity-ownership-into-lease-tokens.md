---
id: '1632'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-lease-owned-capacity
entry: acquire
nodes:
  acquire: { kind: start, label: "Acquire idle or fresh backend" }
  lease: { kind: process, label: "Lease owns permit and active RAII token" }
  relay: { kind: process, label: "Relay session or one transaction" }
  return: { kind: process, label: "Drop active token then reset stream" }
  idle: { kind: process, label: "Park stream and permit in idle collection" }
  close: { kind: process, label: "Drop stream and permit then wake waiters" }
  dropped: { kind: terminal, label: "Unreleased lease RAII frees capacity once" }
edges:
  - { from: acquire, to: lease }
  - { from: lease, to: relay }
  - { from: relay, to: return, label: "ReturnToIdle" }
  - { from: return, to: idle, label: "reset ReadyForQuery idle" }
  - { from: return, to: close, label: "reset failure" }
  - { from: relay, to: close, label: "Close" }
  - { from: lease, to: dropped, label: "lease dropped" }
---
flowchart LR
  acquire([acquire]) --> lease[lease owns permit + active token]
  lease --> relay[relay]
  relay -->|return| return[drop active token; reset]
  return -->|clean| idle[park stream + permit]
  return -->|bad| close[drop stream + permit]
  relay -->|close| close
  lease -->|unreleased drop| dropped([free once + notify])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-lease-owned-capacity
    impl_mode: hand-written
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: pgpool-lease-owned-capacity
    impl_mode: hand-written
  - path: apps/pgpool/src/proxy/session.rs
    action: modify
    section: pgpool-lease-owned-capacity
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-lease-owned-capacity
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-lease-owned-capacity
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-lease-owned-capacity-verification
requirements:
  capacity_raii:
    id: R1
    text: "Active and idle physical backend connections own exactly one capacity permit, and dropping an unreleased lease returns it once and wakes waiters."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool
  pool_modes:
    id: R4
    text: "Transaction reset isolation, failed-reset disposal, and whole-session lease semantics remain correct without the shared outstanding map."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes
---
flowchart TD
    r1[R1 capacity raii] --> cargo_test_p_pgpool_test_pool[cargo test -p pgpool --test pool]
    r4[R4 pool modes] --> cargo_test_p_pgpool_test_pool_test_pool_modes[cargo test -p pgpool --test pool --test pool_modes]
```
