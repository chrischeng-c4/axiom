---
id: '1695'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-reset-reader-reuse
entry: leg_ready
nodes:
  leg_ready: { kind: start, label: "Transaction relay has validated ReadyForQuery Idle" }
  buffered: { kind: decision, label: "Backend reader has residual bytes" }
  close_residual: { kind: terminal, label: "Close; residual backend bytes never cross reset ownership" }
  reunite: { kind: process, label: "Reunite the two backend TCP halves" }
  reset_same_reader: { kind: process, label: "Send static DISCARD ALL and read response with transferred reader" }
  reset_new_reader: { kind: process, label: "Generic pool release constructs its existing reset reader" }
  reset_ready: { kind: decision, label: "Reset reaches valid ReadyForQuery Idle before timeout" }
  park: { kind: terminal, label: "Park same stream and permit in idle" }
  close_reset: { kind: terminal, label: "Shutdown and free permit" }
edges:
  - { from: leg_ready, to: buffered }
  - { from: buffered, to: close_residual, label: "yes" }
  - { from: buffered, to: reunite, label: "no" }
  - { from: reunite, to: reset_same_reader }
  - { from: reset_same_reader, to: reset_ready }
  - { from: reset_new_reader, to: reset_ready }
  - { from: reset_ready, to: park, label: "yes" }
  - { from: reset_ready, to: close_reset, label: "no" }
---
flowchart LR
  ready([validated transaction ReadyForQuery Idle]) --> bytes{reader buffer empty?}
  bytes -->|no| reject([close backend; never reset or reuse])
  bytes -->|yes| reunite[reunite backend halves]
  reunite --> same[send DISCARD ALL; reuse reader]
  same --> valid{valid reset Idle before timeout?}
  generic[generic release] --> fresh[construct fresh reader]
  fresh --> valid
  valid -->|yes| idle([park stream and permit])
  valid -->|no| close([shutdown stream and free permit])
```

### Ownership rules

- `FrameReader` ownership transfers only after the transaction relay has already emitted and validated `ReadyForQuery(Idle)` and its internal buffer is empty. A residual byte is a protocol-boundary failure, not an opportunity to reset or reuse the connection.
- The transferred reader is used only for the reset response. It keeps the existing bounded frame decoding, timeout, malformed-frame rejection, and valid-Idle condition; it cannot make reset payloads opaque or relax `DISCARD ALL`.
- Generic `BackendPool::release` callers do not supply a reader and therefore retain the current fresh-reader reset path. Both routes preserve the existing stream/permit transition: only success parks the exact stream in `idle`; any failure shuts it down and releases capacity.
- The transaction lease's capacity guard remains alive through the explicit pool release. Reusing a reader adds no permit, waiter, or scheduling ownership path.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-reset-reader-reuse
    impl_mode: hand-written
    reason: Add an internal reset route that consumes a caller-supplied drained backend reader while retaining the public generic release fallback.
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: pgpool-reset-reader-reuse
    impl_mode: hand-written
    reason: Hand the transaction reader to reset only after a validated idle result and reunite failure handling.
  - path: apps/pgpool/src/wire/reader.rs
    action: modify
    section: pgpool-reset-reader-reuse
    impl_mode: hand-written
    reason: Provide an explicit drained-buffer predicate without exposing mutable parser internals.
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-reset-reader-reuse
    impl_mode: hand-written
    reason: Preserve generic reset and failure-close regression coverage.
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-reset-reader-reuse
    impl_mode: hand-written
    reason: Exercise contended transaction reset isolation and one-backend capacity through the reused-reader path.
  - path: apps/pgpool/tests/wire_codec.rs
    action: modify
    section: pgpool-reset-reader-reuse
    impl_mode: hand-written
    reason: Test the reader-drained precondition and state-validating reset boundary.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-reset-reader-reuse-verification
requirements:
  drained_reader_only:
    id: R1
    text: "Only a transaction reader that has reached ReadyForQuery Idle with no residual buffered bytes may be transferred into reset validation."
    kind: regression
    risk: high
    verify: wire_codec::transaction_reset_reader_reuse_requires_drained_idle_reader
  generic_fallback:
    id: R2
    text: "Generic BackendPool release retains its fresh-reader reset path and still closes on malformed or failed reset response."
    kind: regression
    risk: high
    verify: pool::release_return_to_idle_closes_connection_when_reset_fails
  peer_gate:
    id: R4
    text: "The unchanged competitor benchmark remains the sole success gate; meter is diagnostic only and a first valid loss reverts production code."
    kind: integration
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool
  transaction_isolation:
    id: R3
    text: "A contended next transaction observes DISCARD ALL isolation and the configured one-backend capacity bound after reader reuse."
    kind: integration
    risk: high
    verify: pool_modes::transaction_mode_reused_reset_reader_preserves_isolation_and_capacity
---
flowchart TD
    r1[R1 drained reader only] --> wire_codec_transaction_reset_reader_reuse_requires_drained_idle_reader[wire_codec::transaction_reset_reader_reuse_requires_drained_idle_reader]
    r2[R2 generic fallback] --> pool_release_return_to_idle_closes_connection_when_reset_fails[pool::release_return_to_idle_closes_connection_when_reset_fails]
    r3[R3 transaction isolation] --> pool_modes_transaction_mode_reused_reset_reader_preserves_isolation_and_capacity[pool_modes::transaction_mode_reused_reset_reader_preserves_isolation_and_capacity]
    r4[R4 peer gate] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_pgpool_bin_target_release_pgpool[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool]
```
