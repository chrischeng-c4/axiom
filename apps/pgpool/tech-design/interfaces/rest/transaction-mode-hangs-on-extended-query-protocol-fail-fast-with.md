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
