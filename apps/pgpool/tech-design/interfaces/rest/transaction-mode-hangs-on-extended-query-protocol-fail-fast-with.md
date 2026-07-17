---
id: '1876'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: transaction-extended-protocol-error-contract
entry: extended_frame
nodes:
  extended_frame: { kind: start, label: "P B D E H C or S frontend frame" }
  both_engines: { kind: process, label: "legacy and reactor detect before backend relay" }
  error: { kind: process, label: "emit FATAL 0A000 ErrorResponse" }
  flush_close: { kind: terminal, label: "flush error then close client and lease" }
edges:
  - { from: extended_frame, to: both_engines }
  - { from: both_engines, to: error }
  - { from: error, to: flush_close }
---
flowchart TD
  extended_frame[extended frontend tag] --> both_engines[detect in legacy and reactor]
  both_engines --> error[FATAL 0A000 unsupported message]
  error --> flush_close[flush then close]
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
