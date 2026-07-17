---
id: '1877'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lossless-wire-relay-contract
entry: frame
nodes:
  frame: { kind: start, label: "complete bounded PostgreSQL frame" }
  accepted: { kind: process, label: "validate then relay byte-for-byte" }
  malformed: { kind: terminal, label: "reject malformed or over-limit frame" }
  cancel: { kind: terminal, label: "reject CancelRequest deliberately" }
edges:
  - { from: frame, to: accepted, label: covered tag }
  - { from: frame, to: malformed, label: invalid }
  - { from: frame, to: cancel, label: CancelRequest }
---
flowchart TD
  frame[complete frame] -->|covered tag| accepted[validate and relay bytes]
  frame -->|invalid| malformed[bounded rejection]
  frame -->|cancel| cancel[explicit unsupported branch]
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
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lossless-wire-relay-tag-coverage-verification
requirements:
  session_losslessness:
    id: R2
    text: "Session mode preserves COPY, LISTEN/NOTIFY, empty-query, and extended-query exchanges instead of disconnecting on an unknown tag."
    kind: functional
    risk: high
    verify: cargo test -p pgpool --test session_proxy session_mode_relays_extended_copy_notify_and_empty_query
  tag_validation:
    id: R1
    text: "Every newly accepted frontend and backend tag has well-formed relay coverage plus malformed and oversize rejection coverage."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test wire_codec relay_accepts_extended_copy_notify_and_empty_frames
  transaction_definition:
    id: R3
    text: "Transaction relay validation accepts covered frames up to its explicit extended-protocol fail-fast boundary rather than failing with UnknownTag."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test wire_codec transaction_relay_covered_tags_are_not_unknown
---
flowchart TD
    r1[R1 tag validation] --> cargo_test_p_pgpool_test_wire_codec_relay_accepts_extended_copy_notify_and_empty_frames[cargo test -p pgpool --test wire_codec relay_accepts_extended_copy_notify_and_empty_frames]
    r2[R2 session losslessness] --> cargo_test_p_pgpool_test_session_proxy_session_mode_relays_extended_copy_notify_and_empty_query[cargo test -p pgpool --test session_proxy session_mode_relays_extended_copy_notify_and_empty_query]
    r3[R3 transaction definition] --> cargo_test_p_pgpool_test_wire_codec_transaction_relay_covered_tags_are_not_unknown[cargo test -p pgpool --test wire_codec transaction_relay_covered_tags_are_not_unknown]
```
