---
id: '1623'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-reset-reader-reuse-contract
entry: ready
nodes:
  ready: { kind: start, label: "Leg ReadyForQuery Idle received" }
  handoff: { kind: process, label: "Reunite stream and hand reader to pool reset" }
  reset: { kind: process, label: "Static DISCARD ALL then same reader validates backend response" }
  outcome: { kind: terminal, label: "Idle on success or close on failure" }
edges:
  - { from: ready, to: handoff }
  - { from: handoff, to: reset }
  - { from: reset, to: outcome }
---
flowchart LR
  ready([leg ReadyForQuery Idle]) --> handoff[stream plus reader handoff]
  handoff --> reset[static reset and same reader]
  reset --> outcome([idle or close])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-reset-reader-reuse-contract
    impl_mode: hand-written
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: pgpool-reset-reader-reuse-contract
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-reset-reader-reuse-contract
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
