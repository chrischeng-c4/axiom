---
id: '1681'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-readiness-gated-idle-liveness-contract
entry: idle_tuple
nodes:
  idle_tuple: { kind: start, label: "Idle tuple exclusively owns a reset-clean stream and its permit." }
  try_io: { kind: process, label: "Ask Tokio's READABLE registration to invoke one socket peek only if it considers the descriptor ready." }
  classify: { kind: decision, label: "Classify no-ready/stale WouldBlock, positive peek, EOF, and other error." }
  lease: { kind: terminal, label: "Return unchanged stream and transfer permit to outstanding." }
  discard: { kind: terminal, label: "Drop tuple and continue existing acquire fallback." }
edges:
  - { from: idle_tuple, to: try_io }
  - { from: try_io, to: classify }
  - { from: classify, to: lease, label: "WouldBlock or bytes > 0" }
  - { from: classify, to: discard, label: "EOF or other error" }
---
flowchart TD
  idle_tuple([idle tuple]) --> try_io[try_io READABLE + conditional MSG_PEEK]
  try_io --> classify{outcome}
  classify -->|not ready/stale WouldBlock| lease([reuse unchanged])
  classify -->|queued bytes| lease
  classify -->|EOF/error| discard([drop then retry])
```

### Contract invariants

- `TcpStream::try_io(Interest::READABLE, closure)` is the sole readiness authority. When it returns `WouldBlock` before executing the closure, the stream is live and no syscall ran.
- The closure uses an external socket facade rather than a Tokio stream method, exactly as Tokio requires for `try_io`; it performs only one READABLE operation.
- Closure `WouldBlock` is returned only after a real peek sees stale readiness; Tokio can clear that stale read bit. This still yields a live stream.
- Positive peek results preserve bytes; zero results and non-`WouldBlock` errors discard the tuple.

### Compatibility

This is internal behavior only. It retains the established downstream PostgreSQL wire semantics and all pool API/error shapes. It differs from #1680 by making the common non-readable idle state a runtime-only decision rather than a kernel syscall.
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/Cargo.toml
    action: modify
    section: pgpool-readiness-gated-idle-liveness
    impl_mode: hand-written
    reason: Declare the safe socket facade used solely inside Tokio's readiness-gated closure.
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-readiness-gated-idle-liveness
    impl_mode: hand-written
    reason: Replace timer liveness with Tokio READABLE gating and conditional non-consuming socket peek classification.
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-readiness-gated-idle-liveness
    impl_mode: hand-written
    reason: Cover no-readiness reuse, ready EOF rejection, and preserved queued protocol bytes.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-readiness-gated-idle-liveness-verification
requirements:
  nonready_reuse:
    id: R1
    text: "An idle backend with no registered readable state is reused without a timer-backed wait."
    kind: regression
    risk: high
    verify: pool::acquire_reuses_idle_connection_after_liveness_check_passes
  performance_evidence:
    id: AC4
    text: "Meter informs bottleneck diagnosis only; retention requires clean unsampled wins under the fixed PgBouncer benchmark contract."
    kind: e2e
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh
  pool_modes_unchanged:
    id: R4
    text: "Pool mode, reset, capacity, and replay contracts remain unchanged around the new liveness gate."
    kind: integration
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes --test trust_startup_replay
  readable_bytes_preserved:
    id: R3
    text: "Read-ready queued bytes are observed only with MSG_PEEK and are preserved for the lease holder."
    kind: regression
    risk: high
    verify: pool::acquire_liveness_peek_preserves_queued_backend_bytes
  ready_eof_discard:
    id: R2
    text: "Read-ready peer EOF is detected before a lease is returned and acquisition retries with a fresh backend."
    kind: regression
    risk: high
    verify: pool::acquire_drops_dead_idle_connection_and_retries
---
flowchart TD
    r1[R1 nonready reuse] --> pool_acquire_reuses_idle_connection_after_liveness_check_passes[pool::acquire_reuses_idle_connection_after_liveness_check_passes]
    r2[R2 ready eof discard] --> pool_acquire_drops_dead_idle_connection_and_retries[pool::acquire_drops_dead_idle_connection_and_retries]
    r3[R3 readable bytes preserved] --> pool_acquire_liveness_peek_preserves_queued_backend_bytes[pool::acquire_liveness_peek_preserves_queued_backend_bytes]
    ac4[AC4 performance evidence] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh]
    r4[R4 pool modes unchanged] --> cargo_test_p_pgpool_test_pool_test_pool_modes_test_trust_startup_replay[cargo test -p pgpool --test pool --test pool_modes --test trust_startup_replay]
```
