---
id: '1624'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-direct-reader-buffer-read-contract
entry: need_bytes
nodes:
  need_bytes: { kind: start, label: "FrameReader has no complete frame" }
  append: { kind: process, label: "Async read_buf appends directly to owned BytesMut" }
  parser: { kind: process, label: "Existing take_frame and validation" }
  result: { kind: terminal, label: "Frame, EOF, or mapped I/O error" }
edges:
  - { from: need_bytes, to: append }
  - { from: append, to: parser }
  - { from: parser, to: result }
---
flowchart LR
  need_bytes([incomplete frame]) --> append[direct append to BytesMut]
  append --> parser[unchanged parser validation]
  parser --> result([frame EOF or I/O error])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/wire/reader.rs
    action: modify
    section: pgpool-direct-reader-buffer-read-contract
    impl_mode: hand-written
  - path: apps/pgpool/src/proxy/relay.rs
    action: modify
    section: pgpool-direct-reader-buffer-read-contract
    impl_mode: hand-written
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-direct-reader-buffer-read-contract
    impl_mode: hand-written
  - path: apps/pgpool/tests/wire_codec.rs
    action: modify
    section: pgpool-direct-reader-buffer-read-contract
    impl_mode: hand-written
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-direct-reader-buffer-read-contract-verification
requirements:
  pool_safety:
    id: R2
    text: "Reset and transaction relay behavior remain session-isolated with no changed failure path."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --lib --test pool --test pool_modes
  wire_equivalence:
    id: R1
    text: "Direct buffer reads preserve exact frame bytes, split-frame recovery, bounds, and EOF semantics."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test wire_codec
---
flowchart TD
    r1[R1 wire equivalence] --> cargo_test_p_pgpool_test_wire_codec[cargo test -p pgpool --test wire_codec]
    r2[R2 pool safety] --> cargo_test_p_pgpool_lib_test_pool_test_pool_modes[cargo test -p pgpool --lib --test pool --test pool_modes]
```
