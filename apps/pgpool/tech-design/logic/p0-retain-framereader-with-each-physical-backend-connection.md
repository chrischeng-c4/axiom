---
id: '1625'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-physical-backend-reader-contract
entry: fresh
nodes:
  fresh: { kind: start, label: "Fresh connection creates stream and reader" }
  active: { kind: process, label: "Lease reader and stream together" }
  reset: { kind: process, label: "Same reader consumes reset response to ReadyForQuery" }
  idle: { kind: process, label: "Store clean reader and stream together" }
  close: { kind: terminal, label: "Close both on error EOF or failed reset" }
edges:
  - { from: fresh, to: active }
  - { from: active, to: reset }
  - { from: reset, to: idle }
  - { from: active, to: close }
  - { from: reset, to: close }
---
flowchart LR
  fresh([fresh stream and reader]) --> active[lease together]
  active --> reset[same reader consumes reset]
  reset --> idle[store clean reader and stream]
  active --> close([close on error])
  reset --> close
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-physical-backend-reader-contract
    impl_mode: hand-written
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: pgpool-physical-backend-reader-contract
    impl_mode: hand-written
  - path: apps/pgpool/src/proxy/session.rs
    action: modify
    section: pgpool-physical-backend-reader-contract
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-physical-backend-reader-contract
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-physical-backend-reader-contract
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
