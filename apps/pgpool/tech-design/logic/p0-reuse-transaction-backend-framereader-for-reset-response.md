---
id: '1623'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-reset-reader-reuse
entry: transaction_ready
nodes:
  transaction_ready: { kind: start, label: "Transaction reader sees ReadyForQuery Idle" }
  release: { kind: process, label: "Pass drained reader with released backend stream" }
  reset: { kind: process, label: "Write DISCARD ALL and validate response using same reader" }
  idle: { kind: terminal, label: "Reuse backend only after reset ReadyForQuery" }
edges:
  - { from: transaction_ready, to: release }
  - { from: release, to: reset }
  - { from: reset, to: idle }
---
flowchart LR
  transaction_ready([transaction ReadyForQuery Idle]) --> release[release stream plus drained reader]
  release --> reset[DISCARD ALL using same reader]
  reset --> idle([safe idle reuse])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-reset-reader-reuse
    impl_mode: hand-written
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: pgpool-reset-reader-reuse
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-reset-reader-reuse
    impl_mode: hand-written
```
