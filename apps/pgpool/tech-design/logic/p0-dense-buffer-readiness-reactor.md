---
id: '1753'
summary: Make transaction pooling a single-owner readiness loop with dense socket state, reusable parser buffers, vectored output, and optimistic writes while preserving reset-before-reuse isolation.
capability_refs:
  - id: competitor-performance
    role: primary
    gap: external-pooler-comparison
    claim: external-pooler-comparison
    coverage: full
    rationale: "Closes the local PgBouncer comparison gap with six eligible counterbalanced release wins on the fixed 64-client, 16-backend, simple-protocol profile."
  - id: postgres-pooler-core
    role: contributes
    gap: transaction-session-pool-modes
    claim: transaction-session-pool-modes
    coverage: partial
    rationale: "Changes only the transaction data-plane engine; session mode and the existing transaction isolation contract remain unchanged."
fill_sections: [logic, changes, unit-test]
---

# P0 dense-buffer readiness reactor

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-dense-buffer-readiness-reactor
entry: accept_handoff
nodes:
  accept_handoff:
    kind: start
    label: "The Tokio listener admits a frontend and hands its nonblocking socket and connection permit to one readiness owner."
  dense_slot:
    kind: process
    label: "Register frontend and backend sockets in token-indexed dense slots; one reactor owns all lease and readiness transitions."
  parse_frontend:
    kind: process
    label: "Read directly into the persistent FrameReader buffer and validate complete PostgreSQL frames until WouldBlock."
  assign_backend:
    kind: decision
    label: "Is a reset-clean idle backend available for this waiting transaction?"
  wait_fifo:
    kind: process
    label: "Queue the client FIFO with an O(1), monotonic acquire deadline and apply socket backpressure."
  relay:
    kind: process
    label: "Move validated Bytes into a reusable output queue and write_vectored in the same readiness batch; wait for WRITABLE only after WouldBlock."
  transaction_status:
    kind: decision
    label: "What TransactionStatus arrived in ReadyForQuery?"
  keep_lease:
    kind: process
    label: "For InTransaction or FailedTransaction, keep the same backend owner and continue relaying."
  reset:
    kind: process
    label: "For Idle, send DISCARD ALL and mark the backend Resetting; it is not assignable yet."
  reuse:
    kind: process
    label: "Only the reset ReadyForQuery(Idle) moves the backend to clean idle and directly assigns the oldest live waiter."
  close:
    kind: terminal
    label: "EOF, malformed wire data, I/O failure, timeout, or drain completion removes both ownership records and releases the frontend permit."
edges:
  - from: accept_handoff
    to: dense_slot
  - from: dense_slot
    to: parse_frontend
  - from: parse_frontend
    to: assign_backend
  - from: assign_backend
    to: wait_fifo
    label: "none clean"
  - from: assign_backend
    to: relay
    label: "clean backend"
  - from: wait_fifo
    to: relay
    label: "reset completes"
  - from: relay
    to: transaction_status
  - from: transaction_status
    to: keep_lease
    label: "in transaction or failed"
  - from: keep_lease
    to: relay
  - from: transaction_status
    to: reset
    label: "idle"
  - from: reset
    to: reuse
    label: "reset ReadyForQuery idle"
  - from: reuse
    to: relay
    label: "waiter exists"
  - from: reuse
    to: parse_frontend
    label: "backend parked"
  - from: parse_frontend
    to: close
    label: "terminal input"
---
flowchart TD
    accept_handoff([Tokio accept hands socket and permit to reactor]) --> dense_slot[Register token-indexed client/backend slots]
    dense_slot --> parse_frontend[Read directly into persistent parser buffer]
    parse_frontend --> assign_backend{Reset-clean backend available?}
    assign_backend -->|no| wait_fifo[Queue FIFO with monotonic deadline]
    assign_backend -->|yes| relay[Validate and vectored-write in the same readiness batch]
    wait_fifo -->|reset completes| relay
    relay --> transaction_status{ReadyForQuery status}
    transaction_status -->|InTransaction or Failed| keep_lease[Keep backend owner]
    keep_lease --> relay
    transaction_status -->|Idle| reset[Send DISCARD ALL; mark Resetting]
    reset -->|reset ReadyForQuery Idle| reuse[Assign oldest waiter or park clean idle]
    reuse --> relay
    parse_frontend -->|EOF, malformed, timeout, drain| close([Remove ownership and release permit])
```

The reactor is the default transaction engine. `PGPOOL_TRANSACTION_ENGINE=legacy`
is an explicit operational rollback only; failure to start the reactor also
falls back to the existing Tokio handler. Session pooling never enters this
reactor.

The ownership invariant is unchanged: a backend may be assigned only from the
clean-idle queue. A transaction `ReadyForQuery(Idle)` starts `DISCARD ALL`; it
does not make the backend reusable. Only the reset response does. Pipelined
frontend frames remain attached to their client and wait behind that boundary.

Tokens index dense `Vec<Option<T>>` slots directly. Closed tokens are retired
until the current readiness snapshot is fully consumed, then recycled from a
free list; this bounds slot growth without letting stale events target a new
socket. Wait deadlines share one configured duration, so insertion order is
deadline order and a lazy FIFO can discard stale epochs in O(1). Parser buffers
read directly from nonblocking sockets. Validated raw `Bytes` move into a
reusable queue and a bounded `write_vectored` flush runs at the end of the
current readiness batch; WRITABLE interest is retained only on `WouldBlock`.

This P0 preserves pgpool's existing transaction-relay protocol boundary: the
competitive profile and ownership proof use simple-query command cycles.
Cross-lease extended-protocol prepared-statement identity/remapping remains a
separate wire capability under #1287; this reactor does not claim partial
support for it.

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/src/pool/reactor/state.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: Own dense client/backend phases, clean-idle and waiter FIFO queues, and reset-before-reuse transitions.
  - path: apps/pgpool/src/pool/reactor/runtime.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: Own mio readiness, token slots, monotonic deadlines, reusable buffers, vectored output, optimistic flush, startup replay, and drain lifetimes.
  - path: apps/pgpool/src/pool/transaction.rs
    action: update
    section: logic
    impl_mode: hand-written
    reason: Make the reactor the transaction-mode default while retaining an explicit legacy rollback.
  - path: apps/pgpool/src/wire/reader.rs
    action: update
    section: logic
    impl_mode: hand-written
    reason: Read a synchronous nonblocking socket directly into the bounded parser buffer.
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: update
    section: logic
    impl_mode: hand-written
    reason: Publish reactor-owned active and idle counts through the existing pool stats API.
  - path: libs/tcp-server/src/lib.rs
    action: update
    section: logic
    impl_mode: hand-written
    reason: Retain handler-owned readiness resources through connection drain.
  - path: apps/pgpool/tests/pool_modes.rs
    action: update
    section: unit-test
    impl_mode: hand-written
    reason: Preserve backend reuse, contention, saturation, stats, and reset-isolation coverage on the default reactor, and prove failed backend connects release frontend capacity.
  - path: apps/pgpool/tests/pgbouncer_benchmark.rs
    action: update
    section: unit-test
    impl_mode: hand-written
    reason: Pin the host-level serialization rule alongside the immutable counterbalanced profile contract.
  - path: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/README.md
    action: update
    section: e2e-test
    impl_mode: hand-written
    reason: Document that a peer verdict requires the serialized, uncontended runner rather than overlapping host load.
  - path: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh
    action: update
    section: e2e-test
    impl_mode: hand-written
    reason: Keep the fixed counterbalanced PgBouncer comparison contract while serializing peer runs so competing host workloads cannot invalidate a release verdict.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-dense-buffer-readiness-reactor-verification
requirements:
  r1_reset_isolation:
    id: R1
    text: "A backend cannot serve another owner until DISCARD ALL returns ReadyForQuery(Idle), including pipelined and contended clients."
    kind: integration
    risk: high
    verify: pool_modes::reset_between_owners_prevents_session_state_leak_across_transaction_leases
  r2_reuse_and_capacity:
    id: R2
    text: "Sequential, contended, replayed-startup, saturation-timeout, refused-backend, and churn paths preserve reuse, capacity, typed errors, stats, and drain behavior."
    kind: integration
    risk: high
    verify: cargo test -p pgpool
  ac1_pgbouncer_win:
    id: AC1
    text: "On the unchanged 64-client, 16-backend, 30-second simple-protocol select-only ABBA profile, all clients complete without pgbench errors and both orders favor pgpool in at least three independent clean release runs."
    kind: performance
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --workload select-only
---
flowchart TD
    r1[R1 reset isolation] --> pool_modes[pool_modes reset and contention tests]
    r2[R2 reuse capacity stats drain] --> full_suite[cargo test -p pgpool]
    ac1[AC1 three independent eligible wins] --> abba[fixed PgBouncer ABBA runner]
```
