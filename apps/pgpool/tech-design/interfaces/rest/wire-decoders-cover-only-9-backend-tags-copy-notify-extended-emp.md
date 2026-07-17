---
id: '1877'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lossless-wire-relay-tag-coverage
entry: bounded_frame
nodes:
  bounded_frame: { kind: start, label: "length-bounded frontend or backend frame" }
  classify: { kind: decision, label: "known control frame or opaque relay frame" }
  validate: { kind: process, label: "validate minimum structure and configured bounds" }
  relay: { kind: process, label: "preserve exact bytes across session and relay paths" }
  cancel: { kind: terminal, label: "deliberately reject unsupported CancelRequest" }
edges:
  - { from: bounded_frame, to: classify }
  - { from: classify, to: validate }
  - { from: validate, to: relay }
  - { from: classify, to: cancel, label: cancel }
---
flowchart TD
  bounded_frame[bounded wire frame] --> classify{recognized control frame?}
  classify --> validate[validate structure and limits]
  validate --> relay[verbatim relay]
  classify -->|CancelRequest| cancel[deliberate rejection]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/wire/backend.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: BackendMessage::decode
  - path: apps/pgpool/src/wire/frontend.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: FrontendMessage::decode
  - path: apps/pgpool/src/wire/reader.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: validate_backend_relay
  - path: apps/pgpool/tests/wire_codec.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: frontend_extended_query_round_trip
  - path: apps/pgpool/tests/session_proxy.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: real_postgres_session_connects_queries_and_disconnects_cleanly
```
