---
id: '1620'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-static-discard-all-frame
entry: release
nodes:
  release: { kind: start, label: "Backend release to idle" }
  frame: { kind: process, label: "Write byte-exact static DISCARD ALL Query frame" }
  reader: { kind: process, label: "Use existing FrameReader until ReadyForQuery" }
  idle: { kind: terminal, label: "Reset backend eligible for reuse" }
edges:
  - { from: release, to: frame }
  - { from: frame, to: reader }
  - { from: reader, to: idle }
---
flowchart LR
  release([release backend]) --> frame[static DISCARD ALL frame]
  frame --> reader[existing FrameReader to ReadyForQuery]
  reader --> idle([safe idle backend])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-static-discard-all-frame
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-static-discard-all-frame
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-static-discard-all-frame-verification
requirements:
  isolation:
    id: R2
    text: "Reset failure closes the backend and transaction leases remain session-isolated."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes
  wire_equivalence:
    id: R1
    text: "The static reset bytes are exactly the PostgreSQL simple Query frame for DISCARD ALL."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool
---
flowchart TD
    r1[R1 wire equivalence] --> cargo_test_p_pgpool_test_pool[cargo test -p pgpool --test pool]
    r2[R2 isolation] --> cargo_test_p_pgpool_test_pool_test_pool_modes[cargo test -p pgpool --test pool --test pool_modes]
```
