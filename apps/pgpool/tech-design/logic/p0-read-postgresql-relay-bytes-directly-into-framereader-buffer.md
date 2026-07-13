---
id: '1624'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-direct-reader-buffer-read
entry: relay
nodes:
  relay: { kind: start, label: "Relay needs more bytes" }
  read_buf: { kind: process, label: "Append socket bytes directly into FrameReader BytesMut" }
  validate: { kind: process, label: "Existing frame bounds and structural validation" }
  frame: { kind: terminal, label: "Validated raw frame or EOF/error" }
edges:
  - { from: relay, to: read_buf }
  - { from: read_buf, to: validate }
  - { from: validate, to: frame }
---
flowchart LR
  relay([need relay bytes]) --> read_buf[direct read_buf into FrameReader]
  read_buf --> validate[existing bounds and validation]
  validate --> frame([validated frame or EOF/error])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/wire/reader.rs
    action: modify
    section: pgpool-direct-reader-buffer-read
    impl_mode: hand-written
  - path: apps/pgpool/src/proxy/relay.rs
    action: modify
    section: pgpool-direct-reader-buffer-read
    impl_mode: hand-written
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-direct-reader-buffer-read
    impl_mode: hand-written
  - path: apps/pgpool/tests/wire_codec.rs
    action: modify
    section: pgpool-direct-reader-buffer-read
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-direct-reader-buffer-read-verification
requirements:
  relay_isolation:
    id: R2
    text: "Transaction relay retains reset boundaries and session-state isolation after using direct reader-buffer reads."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --lib --test pool --test pool_modes
  split_and_eof:
    id: R1
    text: "Direct reads preserve split-frame accumulation, exact raw bytes, clean EOF, and I/O error behavior."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test wire_codec
---
flowchart TD
    r1[R1 split and eof] --> cargo_test_p_pgpool_test_wire_codec[cargo test -p pgpool --test wire_codec]
    r2[R2 relay isolation] --> cargo_test_p_pgpool_lib_test_pool_test_pool_modes[cargo test -p pgpool --lib --test pool --test pool_modes]
```
