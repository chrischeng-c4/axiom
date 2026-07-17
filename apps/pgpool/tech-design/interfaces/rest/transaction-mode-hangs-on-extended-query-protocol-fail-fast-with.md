---
id: '1876'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: transaction-extended-protocol-rejection
entry: frontend_frame
nodes:
  frontend_frame: { kind: start, label: "transaction-mode frontend frame" }
  classify: { kind: decision, label: "is Parse Bind Describe Execute Flush Close or Sync" }
  simple: { kind: process, label: "preserve existing simple-query relay" }
  reject: { kind: process, label: "encode ErrorResponse and stop this client" }
  close: { kind: terminal, label: "clean client and lease close" }
edges:
  - { from: frontend_frame, to: classify }
  - { from: classify, to: simple, label: simple }
  - { from: classify, to: reject, label: extended }
  - { from: reject, to: close }
---
flowchart TD
  frontend_frame[transaction frontend frame] --> classify{extended protocol tag?}
  classify -->|no| simple[preserve simple relay]
  classify -->|yes| reject[ErrorResponse unsupported]
  reject --> close[clean close]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/wire/reader.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: validate_frontend_relay
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: run_transaction_client
  - path: apps/pgpool/src/pool/reactor/runtime.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: handle_client_relay_frame
  - path: apps/pgpool/tests/transaction_extended_protocol.rs
    action: create
    section: unit-test
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: transaction-extended-protocol-rejection-verification
requirements:
  extended_parse_rejected:
    id: R1
    text: "Both transaction engines synthesize the unsupported-extended-protocol ErrorResponse then close when a client sends Parse, rather than waiting for ReadyForQuery."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test transaction_extended_protocol parse_is_rejected_without_hang
  simple_protocol_preserved:
    id: R2
    text: "Existing simple-query transaction traffic remains supported after the extended-protocol boundary check."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool_modes transaction_mode_reuses_backend_for_simple_queries
---
flowchart TD
    r1[R1 extended parse rejected] --> cargo_test_p_pgpool_test_transaction_extended_protocol_parse_is_rejected_without_hang[cargo test -p pgpool --test transaction_extended_protocol parse_is_rejected_without_hang]
    r2[R2 simple protocol preserved] --> cargo_test_p_pgpool_test_pool_modes_transaction_mode_reuses_backend_for_simple_queries[cargo test -p pgpool --test pool_modes transaction_mode_reuses_backend_for_simple_queries]
```
