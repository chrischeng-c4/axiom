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
