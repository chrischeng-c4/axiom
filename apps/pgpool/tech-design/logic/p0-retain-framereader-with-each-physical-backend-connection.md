---
id: '1625'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-physical-backend-reader-lifecycle
entry: connect
nodes:
  connect: { kind: start, label: "Fresh physical backend connect" }
  reader: { kind: process, label: "Create one backend FrameReader with the connection" }
  lease: { kind: process, label: "Lease stream and reader through startup or transaction relay" }
  reset: { kind: process, label: "Same reader validates DISCARD ALL response" }
  idle: { kind: terminal, label: "Return clean stream and reader together to idle" }
edges:
  - { from: connect, to: reader }
  - { from: reader, to: lease }
  - { from: lease, to: reset }
  - { from: reset, to: idle }
---
flowchart LR
  connect([fresh physical backend]) --> reader[one backend FrameReader]
  reader --> lease[lease stream and reader]
  lease --> reset[same reader validates reset]
  reset --> idle([return clean stream and reader])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-physical-backend-reader-lifecycle
    impl_mode: hand-written
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: pgpool-physical-backend-reader-lifecycle
    impl_mode: hand-written
  - path: apps/pgpool/src/proxy/session.rs
    action: modify
    section: pgpool-physical-backend-reader-lifecycle
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-physical-backend-reader-lifecycle
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-physical-backend-reader-lifecycle
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-physical-backend-reader-lifecycle-verification
requirements:
  isolation:
    id: R2
    text: "Reset failure still closes a physical backend and cross-owner state remains isolated after reader reuse."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --lib --test pool --test pool_modes
  reader_lifetime:
    id: R1
    text: "An idle backend retains reader state and transaction reuse uses that reader through reset before safe reuse."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool
---
flowchart TD
    r1[R1 reader lifetime] --> cargo_test_p_pgpool_test_pool[cargo test -p pgpool --test pool]
    r2[R2 isolation] --> cargo_test_p_pgpool_lib_test_pool_test_pool_modes[cargo test -p pgpool --lib --test pool --test pool_modes]
```
