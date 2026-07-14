---
id: '1680'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-nonblocking-idle-liveness-peek
entry: acquire_idle
nodes:
  acquire_idle: { kind: start, label: "Acquire pops one reset-clean idle backend." }
  socket_peek: { kind: process, label: "Perform one synchronous socket-level MSG_PEEK on the already nonblocking Tokio TCP socket; it never consumes a byte or creates a timer." }
  result: { kind: decision, label: "Did the peek return WouldBlock, EOF, another error, or queued bytes?" }
  live_pending: { kind: process, label: "WouldBlock means no pending bytes and the backend is live." }
  live_queued: { kind: process, label: "Queued bytes remain in the socket because MSG_PEEK is non-consuming." }
  lease: { kind: terminal, label: "Move the unchanged stream and its permit to outstanding and return a reused lease." }
  discard: { kind: process, label: "EOF or an I/O error drops the stream and permit, notifies capacity waiters, and retries normal acquisition." }
edges:
  - { from: acquire_idle, to: socket_peek }
  - { from: socket_peek, to: result }
  - { from: result, to: live_pending, label: "WouldBlock" }
  - { from: result, to: live_queued, label: "one or more bytes" }
  - { from: live_pending, to: lease }
  - { from: live_queued, to: lease }
  - { from: result, to: discard, label: "EOF or other error" }
---
flowchart LR
    acquire_idle([pop idle backend]) --> socket_peek[nonblocking MSG_PEEK]
    socket_peek --> result{result}
    result -->|WouldBlock| live_pending[alive, no queued bytes]
    result -->|bytes| live_queued[bytes remain queued]
    live_pending --> lease([reuse unchanged stream])
    live_queued --> lease
    result -->|EOF or error| discard[drop, notify, retry]
```

### Contract invariants

- The liveness probe runs only while the stream is exclusively owned by the idle tuple; normal relay never races this peek.
- `WouldBlock` is the normal idle state and returns a live lease without scheduling or awaiting Tokio I/O.
- A successful peek is `MSG_PEEK`: it reads no protocol byte, so the next relay read observes the same PostgreSQL frame boundary.
- EOF and every read error retain the existing dead-idle disposition: stream and permit are dropped before the next acquisition attempt.
- This changes neither `DISCARD ALL` before idle admission nor semaphore/permit ownership, capacity wakeups, or acquire deadlines.

### Error handling

A closed peer returns zero bytes and is discarded. A non-`WouldBlock` read error is likewise treated as unsafe for reuse. If no idle tuple remains after a discard, acquisition follows the existing fresh-connect or saturated-capacity path; the new probe creates no timer, wakeup, or alternate scheduling path.

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/Cargo.toml
    action: modify
    section: pgpool-nonblocking-idle-liveness-peek
    impl_mode: hand-written
    reason: Declare the direct socket abstraction used to issue a safe non-consuming nonblocking peek.
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-nonblocking-idle-liveness-peek
    impl_mode: hand-written
    reason: Replace zero-timeout async liveness probing with socket-level MSG_PEEK classification while retaining idle ownership and retry semantics.
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-nonblocking-idle-liveness-peek
    impl_mode: hand-written
    reason: Prove no-byte idle reuse, EOF discard-and-retry, and preservation of readable bytes across the liveness probe.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-nonblocking-idle-liveness-peek-verification
requirements:
  competitor_evidence:
    id: AC5
    text: "Meter is diagnostic only; retain a candidate solely after error-free unsampled release benchmark wins against the unchanged PgBouncer contract."
    kind: e2e
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh
  dead_idle_is_discarded:
    id: R2
    text: "EOF and non-WouldBlock read failure on an idle backend drop its stream and permit before acquisition retries."
    kind: regression
    risk: high
    verify: pool::acquire_drops_dead_idle_connection_and_retries
  idle_reuse_without_timer:
    id: R1
    text: "A no-byte idle backend is reused through one synchronous nonblocking socket peek without constructing or awaiting a Tokio timer."
    kind: regression
    risk: high
    verify: pool::acquire_reuses_idle_connection_after_liveness_check_passes
  peek_preserves_protocol_bytes:
    id: R3
    text: "A readable byte observed during idle liveness remains queued for the normal relay after the backend is leased."
    kind: regression
    risk: high
    verify: pool::acquire_liveness_peek_preserves_queued_backend_bytes
  pool_contract_unchanged:
    id: R4
    text: "Session and transaction pool modes retain reset isolation, permit accounting, and retry behavior around the changed liveness probe."
    kind: integration
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes --test trust_startup_replay
---
flowchart TD
    r1[R1 idle reuse without timer] --> pool_acquire_reuses_idle_connection_after_liveness_check_passes[pool::acquire_reuses_idle_connection_after_liveness_check_passes]
    r2[R2 dead idle is discarded] --> pool_acquire_drops_dead_idle_connection_and_retries[pool::acquire_drops_dead_idle_connection_and_retries]
    r3[R3 peek preserves protocol bytes] --> pool_acquire_liveness_peek_preserves_queued_backend_bytes[pool::acquire_liveness_peek_preserves_queued_backend_bytes]
    r4[R4 pool contract unchanged] --> cargo_test_p_pgpool_test_pool_test_pool_modes_test_trust_startup_replay[cargo test -p pgpool --test pool --test pool_modes --test trust_startup_replay]
    ac5[AC5 competitor evidence] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh]
```
