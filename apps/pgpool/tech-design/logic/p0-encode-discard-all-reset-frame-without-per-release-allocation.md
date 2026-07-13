---
id: '1620'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-static-discard-all-frame-contract
entry: reset
nodes:
  reset: { kind: start, label: "Reset required before idle reuse" }
  bytes: { kind: process, label: "Static byte-exact Query frame" }
  response: { kind: process, label: "Existing reader validates ReadyForQuery" }
  reuse: { kind: terminal, label: "Reuse only after successful reset" }
edges:
  - { from: reset, to: bytes }
  - { from: bytes, to: response }
  - { from: response, to: reuse }
---
flowchart LR
  reset([reset required]) --> bytes[static Query bytes]
  bytes --> response[existing response validation]
  response --> reuse([safe reuse])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-static-discard-all-frame-contract
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-static-discard-all-frame-contract
    impl_mode: hand-written
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-static-discard-all-frame-contract-verification
requirements:
  fixed_frame:
    id: R1
    text: "The outbound reset Query bytes remain byte-exact for DISCARD ALL without allocating a per-release message buffer."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool
  safe_reuse:
    id: R2
    text: "A reset backend is reused only after ReadyForQuery and failure still closes it."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes
---
flowchart TD
    r1[R1 fixed frame] --> cargo_test_p_pgpool_test_pool[cargo test -p pgpool --test pool]
    r2[R2 safe reuse] --> cargo_test_p_pgpool_test_pool_test_pool_modes[cargo test -p pgpool --test pool --test pool_modes]
```
