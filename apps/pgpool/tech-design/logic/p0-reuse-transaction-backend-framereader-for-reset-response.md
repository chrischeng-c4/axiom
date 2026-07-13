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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-reset-reader-reuse-verification
requirements:
  reader_handoff:
    id: R1
    text: "Transaction release hands its drained backend reader to reset response validation without changing the reset boundary."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool_modes
  safe_reset:
    id: R2
    text: "Reset bytes, failed-reset close behavior, and cross-owner session isolation remain unchanged."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --lib --test pool --test pool_modes
---
flowchart TD
    r1[R1 reader handoff] --> cargo_test_p_pgpool_test_pool_modes[cargo test -p pgpool --test pool_modes]
    r2[R2 safe reset] --> cargo_test_p_pgpool_lib_test_pool_test_pool_modes[cargo test -p pgpool --lib --test pool --test pool_modes]
```
