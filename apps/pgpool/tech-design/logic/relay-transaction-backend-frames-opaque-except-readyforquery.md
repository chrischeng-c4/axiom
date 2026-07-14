---
id: '1684'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-opaque-backend-transaction-relay
entry: buffered_backend_frame
nodes:
  buffered_backend_frame:
    kind: start
    label: "Established transaction relay has one complete tagged backend frame buffered."
  bounded_envelope:
    kind: decision
    label: "Tagged length is at least four and no greater than max_frame_bytes."
  reject_envelope:
    kind: terminal
    label: "Return FrameError; relay ends and the leased backend closes rather than returning to idle."
  ready_tag:
    kind: decision
    label: "Frame tag is ReadyForQuery."
  validate_ready:
    kind: process
    label: "Require exactly one status byte and map I, T, or E to TransactionStatus."
  reject_ready:
    kind: terminal
    label: "Malformed ReadyForQuery fails before it can change ownership or permit reuse."
  forward_opaque:
    kind: process
    label: "Forward every bounded non-ReadyForQuery frame as the original byte slice without payload parsing."
  forward_ready:
    kind: process
    label: "Forward the validated ReadyForQuery bytes and expose its TransactionStatus."
  relay_more:
    kind: terminal
    label: "Continue the same transaction for T/E, or trigger existing reset-before-reuse only for I."
edges:
  - from: buffered_backend_frame
    to: bounded_envelope
  - from: bounded_envelope
    to: reject_envelope
    label: "invalid or oversized"
  - from: bounded_envelope
    to: ready_tag
    label: "complete bounded frame"
  - from: ready_tag
    to: validate_ready
    label: "Z"
  - from: ready_tag
    to: forward_opaque
    label: "any other backend tag"
  - from: validate_ready
    to: reject_ready
    label: "payload invalid"
  - from: validate_ready
    to: forward_ready
    label: "I, T, or E"
  - from: forward_opaque
    to: relay_more
  - from: forward_ready
    to: relay_more
---
flowchart TD
  frame([complete backend frame]) --> bounds{valid bounded envelope?}
  bounds -->|no| close([FrameError and close lease])
  bounds -->|yes| tag{ReadyForQuery?}
  tag -->|no| opaque[forward original bytes without payload parse]
  tag -->|yes| ready[validate 1-byte I/T/E status]
  ready -->|invalid| reject([FrameError and close lease])
  ready -->|valid| status[forward bytes and record status]
  opaque --> continue([continue relay])
  status --> continue
```

### Contract invariants

- This path begins only after the normal startup/authentication relay has established the backend connection. Startup, authentication, reset, session-mode, and typed codec callers retain their existing full decoder.
- `take_frame` remains the sole envelope authority: every frame has a tagged PostgreSQL envelope, a length of at least four, and a configured maximum before any byte is exposed to the frontend.
- `ReadyForQuery` is the sole backend control frame for transaction ownership. Its payload must be exactly one valid status byte before `TransactionStatus` changes or the idle/reset path can run.
- Any other bounded backend frame is opaque to the transaction relay. Its payload cannot select a lease state, bypass `DISCARD ALL`, or cause a connection to re-enter the idle pool.
- Frontend validation, pipelined-query staging, client-visible frame order, and the existing error/EOF close path do not change.

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/src/wire/reader.rs
    action: modify
    section: pgpool-opaque-backend-transaction-relay
    impl_mode: hand-written
    reason: Keep bounded frame extraction and strict ReadyForQuery state classification while making other established backend relay payloads opaque.
  - path: apps/pgpool/tests/wire_codec.rs
    action: modify
    section: pgpool-opaque-backend-transaction-relay
    impl_mode: hand-written
    reason: Prove non-control backend payloads relay byte-for-byte without structural parsing and that malformed ReadyForQuery still rejects.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-opaque-backend-transaction-relay-verification
requirements:
  opaque_non_control:
    id: R1
    text: "A complete bounded non-ReadyForQuery backend frame preserves its exact bytes and never requires a structurally valid result payload to be relayed."
    kind: regression
    risk: high
    verify: wire_codec::transaction_relay_forwards_bounded_non_control_backend_frame_without_payload_validation
  peer_comparison:
    id: R4
    text: "The unchanged 64-client, 16-backend, simple-protocol comparison runs cleanly; a first valid loss reverts production code and preserves evidence."
    kind: integration
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool
  pool_isolation:
    id: R3
    text: "Existing transaction ownership, reset-before-reuse, and pipelined-query isolation remain intact."
    kind: integration
    risk: high
    verify: cargo test -p pgpool --test wire_codec --test pool --test pool_modes
  ready_control_boundary:
    id: R2
    text: "ReadyForQuery remains the only relay control frame: invalid payload length or status rejects before transaction state or reuse can change."
    kind: regression
    risk: high
    verify: wire_codec::transaction_relay_rejects_malformed_ready_for_query
---
flowchart TD
    r1[R1 opaque non control] --> wire_codec_transaction_relay_forwards_bounded_non_control_backend_frame_without_payload_validation[wire_codec::transaction_relay_forwards_bounded_non_control_backend_frame_without_payload_validation]
    r2[R2 ready control boundary] --> wire_codec_transaction_relay_rejects_malformed_ready_for_query[wire_codec::transaction_relay_rejects_malformed_ready_for_query]
    r3[R3 pool isolation] --> cargo_test_p_pgpool_test_wire_codec_test_pool_test_pool_modes[cargo test -p pgpool --test wire_codec --test pool --test pool_modes]
    r4[R4 peer comparison] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_pgpool_bin_target_release_pgpool[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool]
```
