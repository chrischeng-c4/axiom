---
id: apps-pgpool-backend-pool
summary: Backend connection pool for pgpool - a capacity-bounded (RuntimePlan::max_backend_connections) BackendPool with idle reuse, a non-blocking liveness check on acquire, and a DISCARD ALL reset-between-owners step, dispatched by RuntimePlan::PoolMode into the unchanged WI #1288 SessionHandler (now capacity-bounded through the same pool) for session mode, or a new per-transaction TransactionHandler for transaction mode that leases a backend only for the lifetime of one transaction (tracked via the wire codec's ReadyForQuery/TransactionStatus), plus a bounded-wait-then-typed-error saturation path and a plain Rust pool-stats API for the later admin-plane slice.
capability_refs:
  - id: postgres-pooler-core
    role: primary
    gap: backend-pool-and-reuse
    claim: backend-pool-and-reuse
    coverage: full
    rationale: "Defines and closes the backend-pool-and-reuse work root: the apps/pgpool/src/pool/ BackendPool with capacity-bounded acquire/acquire_fresh, idle reuse with a liveness check, and a DISCARD ALL reset-between-owners step before a connection returns to the idle set, verified by cargo test -p pgpool --test pool_modes plus offline apps/pgpool/tests/pool.rs coverage."
  - id: postgres-pooler-core
    role: primary
    gap: transaction-session-pool-modes
    claim: transaction-session-pool-modes
    coverage: full
    rationale: "Defines and closes the transaction-session-pool-modes work root: RuntimePlan::PoolMode-selected dispatch between the new per-transaction TransactionHandler (lease boundaries tracked by backend ReadyForQuery transitions) and the unchanged WI #1288 SessionHandler (now capacity-bounded through the shared BackendPool), verified by cargo test -p pgpool --test pool_modes."
  - id: long-running-stability
    role: contributes
    gap: pool-leak-and-reuse-longrun
    claim: pool-leak-and-reuse-longrun
    coverage: partial
    rationale: "Starts proving the pool-leak-and-reuse-longrun promise: a 100+ cycle churn test (cargo test -p pgpool --test pool_modes churn_100_cycles_holds_backend_count_stable_no_leak) holds the backend connection count stable with no connection/fd leak; full long-run/backend-restart-safety conformance remains open for the separate drain-and-backend-restart-safety work root, out of this WI's scope."
fill_sections: [logic, state-machine, schema, config, unit-test]
---

# pgpool backend pool — connection reuse and transaction/session pool modes

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-backend-pool-logic-flow
entry: handle_accept
nodes:
  handle_accept:
    kind: start
    label: "PoolHandler::handle (TcpHandler) is invoked for an accepted client; RuntimePlan::PoolMode was fixed once at process start (pgpool serve), so every connection in this process takes either the Session or Transaction branch below"
  frontend_admit:
    kind: decision
    label: "ConnectionBudget::try_acquire() (RuntimePlan::frontend_budget, same primitive WI #1288 already uses) succeeds for this connection?"
  reject_frontend_saturated:
    kind: terminal
    label: "Frontend saturated: write BackendMessage::ErrorResponse (SQLSTATE 53300 too_many_connections) to the client and close the socket — unchanged from WI #1288, shared by both pool modes"
  mode_branch:
    kind: decision
    label: "RuntimePlan::PoolMode (fixed for the process)"
  session_mode_delegate:
    kind: terminal
    label: "Session mode: delegates to the unchanged WI #1288 SessionHandler::run_session pipeline, except connect_backend now calls BackendPool::acquire_fresh() (capacity-bounded by max_backend_connections, R1) instead of a raw TcpStream::connect, and teardown calls BackendPool::release(id, stream, LeaseDisposition::Close) instead of just dropping the socket; the per-message auth-passthrough/relay steps are unchanged and are documented in apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md, not redrawn here"
  txn_admit_handshake:
    kind: process
    label: "Transaction mode: BackendPool::acquire_fresh() dials a brand-new backend connection for this client's own one-time real startup+auth relay (reusing the frame-aware relay_startup/relay_until_ready mechanism from the session-mode proxy), bounded by the shared max_backend_connections capacity (R1)"
  handshake_result:
    kind: decision
    label: "Backend connect succeeded and AuthenticationOk + ReadyForQuery(Idle) were forwarded to the client before any ErrorResponse?"
  reject_backend_unreachable:
    kind: terminal
    label: "acquire_fresh() connect failed/timed out: write ErrorResponse (SQLSTATE 08006 connection_failure), release the frontend permit, close the client socket — same RejectionReason::BackendUnreachable mapping as session mode"
  reject_auth_failed:
    kind: terminal
    label: "Backend emitted ErrorResponse during the admission handshake: forward it to the client verbatim, release the frontend permit, close both sides — same RejectionReason::BackendAuthFailed mapping as session mode"
  release_after_handshake:
    kind: process
    label: "Handshake succeeded: this client is now admitted and vouched-for; the handshake backend is immediately reset (DISCARD ALL) and returned to the shared idle pool via BackendPool::release(id, stream, LeaseDisposition::ReturnToIdle) — the client now holds NO backend lease (R1, R2)"
  await_client_activity:
    kind: process
    label: "Client holds no backend lease; a frontend-role FrameReader fed from the client stream waits for the client's next frame"
  txn_start_detected:
    kind: decision
    label: "Is the observed frontend frame the start of a transaction/first query after ReadyForQuery-idle (any frame other than Terminate), or Terminate/clean EOF?"
  client_terminate_idle:
    kind: terminal
    label: "Client sent Terminate (or closed cleanly) while holding no lease: nothing to release, session ends cleanly"
  acquire_txn_backend:
    kind: process
    label: "BackendPool::acquire(): pop an idle already-authenticated connection and liveness-check it (non-blocking peek read), else fresh-connect if capacity remains, else wait up to acquire_timeout (R1, R3)"
  txn_acquire_result:
    kind: decision
    label: "A lease was returned within acquire_timeout, or the wait timed out?"
  reject_pool_saturated:
    kind: terminal
    label: "Backend pool exhausted for longer than acquire_timeout: write a synthesized PoolRejectionReason::BackendPoolSaturated ErrorResponse (SQLSTATE 53300) to this client only and close its socket; every other admitted client and in-flight transaction lease is unaffected (R3, AC3)"
  relay_transaction:
    kind: process
    label: "Relay frontend<->backend frames verbatim on the leased connection (same decode/re-encode primitives as session mode's bidirectional relay) until the backend's ReadyForQuery reports Idle again, or Terminate/EOF/FrameError ends the leg (R2)"
  relay_end_check:
    kind: decision
    label: "How did the leased transaction's relay end?"
  release_after_transaction:
    kind: process
    label: "Backend reported ReadyForQuery(Idle): reset (DISCARD ALL) and BackendPool::release(id, stream, LeaseDisposition::ReturnToIdle); the client returns to holding no lease and loops back to await_client_activity (R1, R2, AC1, AC2)"
  relay_error_or_eof:
    kind: terminal
    label: "Backend/client EOF or FrameError mid-transaction: the lease is released as BackendPool::release(id, stream, LeaseDisposition::Close) — never returned to idle, since its session state is unknown/unsafe to reuse — and the client socket is closed"
  drain_interaction:
    kind: process
    label: "Concurrently, DrainSignal flips to Draining on SIGTERM/SIGINT (unchanged tcp-server mechanism): the accept loop stops admitting new connections immediately, while an in-flight handshake or transaction lease keeps running unaffected until it ends or TcpServerConfig.drain_timeout elapses, at which point the task is abandoned"
edges:
  - from: handle_accept
    to: frontend_admit
    label: "connection accepted"
  - from: frontend_admit
    to: reject_frontend_saturated
    label: "budget exhausted"
  - from: frontend_admit
    to: mode_branch
    label: "permit acquired"
  - from: mode_branch
    to: session_mode_delegate
    label: "PoolMode::Session"
  - from: mode_branch
    to: txn_admit_handshake
    label: "PoolMode::Transaction"
  - from: txn_admit_handshake
    to: handshake_result
    label: "connect + relay attempted"
  - from: handshake_result
    to: reject_backend_unreachable
    label: "connect failed/timed out"
  - from: handshake_result
    to: reject_auth_failed
    label: "backend ErrorResponse before ReadyForQuery"
  - from: handshake_result
    to: release_after_handshake
    label: "AuthenticationOk + ReadyForQuery forwarded"
  - from: release_after_handshake
    to: await_client_activity
    label: "handshake backend reset + returned to idle"
  - from: await_client_activity
    to: txn_start_detected
    label: "a frontend frame arrives"
  - from: txn_start_detected
    to: client_terminate_idle
    label: "Terminate/clean EOF"
  - from: txn_start_detected
    to: acquire_txn_backend
    label: "any other frame (transaction/query start)"
  - from: acquire_txn_backend
    to: txn_acquire_result
    label: "acquire attempted"
  - from: txn_acquire_result
    to: reject_pool_saturated
    label: "acquire_timeout elapsed, no lease"
  - from: txn_acquire_result
    to: relay_transaction
    label: "lease acquired"
  - from: relay_transaction
    to: relay_end_check
    label: "relay ends"
  - from: relay_end_check
    to: release_after_transaction
    label: "backend ReadyForQuery(Idle)"
  - from: relay_end_check
    to: relay_error_or_eof
    label: "EOF/FrameError mid-transaction"
  - from: release_after_transaction
    to: await_client_activity
    label: "loop: client holds no lease again"
  - from: txn_admit_handshake
    to: drain_interaction
    label: "session task holds a DrainSignal for its whole lifetime"
---
flowchart TD
    handle_accept([PoolHandler::handle invoked; PoolMode fixed at process start]) --> frontend_admit{ConnectionBudget::try_acquire succeeds?}
    frontend_admit -->|budget exhausted| reject_frontend_saturated([Write ErrorResponse 53300, close socket])
    frontend_admit -->|permit acquired| mode_branch{RuntimePlan::PoolMode}
    mode_branch -->|Session| session_mode_delegate([Delegates to WI #1288 SessionHandler, now via BackendPool::acquire_fresh/release])
    mode_branch -->|Transaction| txn_admit_handshake[BackendPool::acquire_fresh for this client's real startup+auth relay]
    txn_admit_handshake --> handshake_result{AuthenticationOk + ReadyForQuery forwarded?}
    handshake_result -->|connect failed/timed out| reject_backend_unreachable([Write ErrorResponse 08006, release permit, close])
    handshake_result -->|backend ErrorResponse| reject_auth_failed([Forward ErrorResponse, release permit, close both sides])
    handshake_result -->|AuthenticationOk + ReadyForQuery| release_after_handshake[Reset DISCARD ALL, return handshake backend to idle pool]
    release_after_handshake --> await_client_activity[Client holds no lease; wait for next frontend frame]
    await_client_activity --> txn_start_detected{Terminate/EOF, or transaction/query start?}
    txn_start_detected -->|Terminate/clean EOF| client_terminate_idle([Session ends cleanly, nothing to release])
    txn_start_detected -->|transaction/query start| acquire_txn_backend[BackendPool::acquire: idle-reuse with liveness check, else fresh-connect, else bounded wait]
    acquire_txn_backend --> txn_acquire_result{Lease acquired within acquire_timeout?}
    txn_acquire_result -->|timed out| reject_pool_saturated([Write synthesized ErrorResponse 53300, close this client only])
    txn_acquire_result -->|acquired| relay_transaction[Relay frontend<->backend frames until ReadyForQuery Idle or Terminate/EOF/FrameError]
    relay_transaction --> relay_end_check{How did the transaction's relay end?}
    relay_end_check -->|ReadyForQuery Idle| release_after_transaction[Reset DISCARD ALL, ReturnToIdle]
    relay_end_check -->|EOF/FrameError| relay_error_or_eof([Release as Close, not returned to idle; close client socket])
    release_after_transaction --> await_client_activity
    txn_admit_handshake -.-> drain_interaction[DrainSignal: accept loop stops on drain, in-flight handshake/transaction keeps running until drain_timeout]
```

## Pool Lease State Machine
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: pgpool-backend-pool-lease-fsm
initial: admitting
nodes:
  admitting:
    kind: initial
    label: "Admitting: ConnectionBudget::try_acquire() just succeeded; BackendPool::acquire_fresh() is dialing this client's own one-time real startup+auth handshake backend (R1)"
  rejected_saturated:
    kind: terminal
    label: "RejectedSaturated: frontend ConnectionBudget exhausted before any backend was touched"
  rejected_backend_unreachable:
    kind: terminal
    label: "RejectedBackendUnreachable: acquire_fresh()'s connect failed or exceeded backend_connect_timeout"
  rejected_auth_failed:
    kind: terminal
    label: "RejectedAuthFailed: backend emitted ErrorResponse during the admission handshake, forwarded to the client verbatim"
  idle_no_lease:
    kind: normal
    label: "IdleNoLease: client is admitted and holds no backend lease — either just after the admission handshake's backend was reset+returned to idle, or after a prior transaction's backend was reset+returned to idle (R1, R2)"
  acquiring_transaction:
    kind: normal
    label: "AcquiringTransaction: the client sent a frontend frame other than Terminate; BackendPool::acquire() is running (idle-reuse-preferring, may wait up to acquire_timeout) (R2, R3)"
  rejected_pool_saturated:
    kind: terminal
    label: "RejectedPoolSaturated: acquire_timeout elapsed with no lease available; a synthesized ErrorResponse (53300) is written to this client only and its socket closed — every other admitted client and in-flight lease is unaffected (R3, AC3)"
  transaction_active:
    kind: normal
    label: "TransactionActive: a backend lease is held; frames relay bidirectionally between this client and its leased backend until the backend's ReadyForQuery reports Idle again, or Terminate/EOF/FrameError ends the leg (R2)"
  draining:
    kind: normal
    label: "Draining: SIGTERM/SIGINT observed (DrainSignal flipped); the accept loop stops admitting new connections, but this client's IdleNoLease/TransactionActive progress continues unaffected, bounded by drain_timeout"
  closed:
    kind: terminal
    label: "Closed: client Terminate while IdleNoLease (nothing to release), or a TransactionActive session ending cleanly (its backend already reset+returned to idle) or in error (its backend released as Close, not returned to idle), or drain_timeout elapsing while Draining (task abandoned)"
edges:
  - from: admitting
    to: rejected_saturated
    event: "ConnectionBudget::try_acquire() returns Err"
  - from: admitting
    to: rejected_backend_unreachable
    event: "acquire_fresh() connect fails or times out"
  - from: admitting
    to: rejected_auth_failed
    event: "backend ErrorResponse observed before ReadyForQuery"
  - from: admitting
    to: idle_no_lease
    event: "AuthenticationOk + ReadyForQuery forwarded; handshake backend reset (DISCARD ALL) and returned to idle"
  - from: idle_no_lease
    to: acquiring_transaction
    event: "a frontend frame other than Terminate arrives (transaction/query start)"
  - from: idle_no_lease
    to: closed
    event: "client sends Terminate or closes cleanly while holding no lease"
  - from: acquiring_transaction
    to: transaction_active
    event: "BackendPool::acquire() returns a lease within acquire_timeout"
  - from: acquiring_transaction
    to: rejected_pool_saturated
    event: "acquire_timeout elapses with no lease available"
  - from: transaction_active
    to: idle_no_lease
    event: "backend ReadyForQuery reports Idle again; reset (DISCARD ALL) and returned to idle"
  - from: transaction_active
    to: closed
    event: "FrameError or EOF on either leg mid-transaction; lease released as Close, not returned to idle"
  - from: idle_no_lease
    to: draining
    event: "DrainSignal flips to Draining (SIGTERM/SIGINT)"
  - from: transaction_active
    to: draining
    event: "DrainSignal flips to Draining (SIGTERM/SIGINT)"
  - from: draining
    to: closed
    event: "session ends normally (same transitions as above), or drain_timeout elapses and the task is abandoned"
---
stateDiagram-v2
    [*] --> admitting
    admitting --> rejected_saturated: ConnectionBudget::try_acquire() returns Err
    admitting --> rejected_backend_unreachable: acquire_fresh() connect fails/times out
    admitting --> rejected_auth_failed: backend ErrorResponse before ReadyForQuery
    admitting --> idle_no_lease: AuthenticationOk + ReadyForQuery forwarded, backend reset+returned to idle
    idle_no_lease --> acquiring_transaction: frontend frame other than Terminate arrives
    idle_no_lease --> closed: client sends Terminate or closes cleanly
    acquiring_transaction --> transaction_active: acquire() returns a lease within acquire_timeout
    acquiring_transaction --> rejected_pool_saturated: acquire_timeout elapses, no lease available
    transaction_active --> idle_no_lease: backend ReadyForQuery reports Idle again, reset+returned to idle
    transaction_active --> closed: FrameError/EOF mid-transaction, lease released as Close
    idle_no_lease --> draining: DrainSignal flips to Draining
    transaction_active --> draining: DrainSignal flips to Draining
    draining --> closed: session ends normally, or drain_timeout elapses (task abandoned)
    rejected_saturated --> [*]
    rejected_backend_unreachable --> [*]
    rejected_auth_failed --> [*]
    rejected_pool_saturated --> [*]
    closed --> [*]
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
title: pgpool backend pool types
definitions:
  PoolConfig:
    type: object
    x-rust-derive: [Debug, Clone]
    required: [endpoint, max_backend_connections, acquire_timeout, backend_connect_timeout, wire]
    description: >-
      Configuration for one BackendPool: the single configured backend
      endpoint (reused from proxy::BackendEndpointConfig), the capacity
      bound sourced from RuntimePlan::max_backend_connections (R1), and the
      timeouts/wire bounds the pool's own connect+relay/reset helpers need.
    properties:
      endpoint:
        x-rust-type: crate::proxy::BackendEndpointConfig
        description: "The single configured backend this pool dials; multiple backend databases/pools keyed by database+user are out of scope for this slice (adapter-boundary epic #1283's seam)."
      max_backend_connections:
        type: integer
        x-rust-type: usize
        description: "Capacity bound shared by both pool modes (idle + active <= this value); sourced from RuntimePlan::max_backend_connections (default 512, R1)."
      acquire_timeout:
        type: string
        x-rust-type: std::time::Duration
        description: "Bounds how long BackendPool::acquire() waits for an idle/freed slot before returning PoolError::Saturated (R3, AC3)."
      backend_connect_timeout:
        type: string
        x-rust-type: std::time::Duration
        description: "Bounds a fresh backend TCP connect from acquire()/acquire_fresh(); mirrors SessionProxyConfig::backend_connect_timeout."
      wire:
        x-rust-type: crate::wire::WireCodecConfig
        description: "Frame bounds for the backend-role FrameReader the pool's own admission-handshake and reset helpers use."

  BackendConnectionId:
    type: integer
    x-rust-type: u64
    x-rust-derive: [Debug, Clone, Copy, PartialEq, Eq, Hash]
    description: >-
      Opaque identity for one physical backend TCP connection, stable
      across idle<->leased transitions; used for tracing and test
      assertions of reuse (e.g. AC1's "far fewer backend connections than
      client count").

  LeaseDisposition:
    type: string
    x-rust-enum: true
    x-rust-derive: [Debug, Clone, Copy, PartialEq, Eq]
    enum: [return_to_idle, close]
    description: >-
      How BackendPool::release() disposes of a returned lease:
      return_to_idle resets the connection (DISCARD ALL) and adds it to the
      shared idle set (R1, R2); close tears the physical connection down
      immediately and frees its capacity slot (session-mode teardown, or
      any lease whose session state is unknown/unsafe to reuse after a
      relay error, or a reset that itself failed).

  BackendLease:
    type: object
    x-rust-derive: [Debug]
    required: [id, fresh, stream]
    description: >-
      One leased physical backend connection returned by
      acquire()/acquire_fresh(): the live socket plus its
      BackendConnectionId and whether this lease required a fresh TCP
      connect.
    properties:
      id:
        $ref: "#/definitions/BackendConnectionId"
      fresh:
        type: boolean
        description: >-
          True when this lease required a brand-new TCP connect
          (acquire_fresh() always; acquire() only when the idle set was
          empty and capacity allowed a new connect) — signals the caller
          that a real startup+auth relay is required before the connection
          can carry client traffic; false means an already-authenticated
          idle connection was reused and only post-auth traffic should be
          relayed.
      stream:
        x-rust-type: tokio::net::TcpStream
        description: "The live backend socket for this lease; the caller splits it (into_split) to run the same frame relay helpers session mode already uses."

  PoolError:
    type: object
    x-rust-enum: true
    x-rust-derive: [Debug, thiserror::Error]
    description: >-
      Error taxonomy from BackendPool::acquire()/acquire_fresh(); every
      variant is handled by the caller writing a typed wire ErrorResponse
      (or forwarding the backend's own) and releasing any frontend permit —
      never a panic, never a silent hang or drop.
    oneOf:
      - title: Saturated
        properties:
          max: { type: integer, x-rust-type: usize }
          waited: { type: string, x-rust-type: std::time::Duration }
        description: "acquire_timeout elapsed with the pool at max_backend_connections and no lease freed (R3, AC3); caller maps this to PoolRejectionReason::BackendPoolSaturated."
      - title: BackendUnreachable
        type: string
        description: "A fresh TCP connect (acquire_fresh(), or acquire() falling back to a fresh connect) failed or exceeded backend_connect_timeout; caller maps this to the existing proxy::RejectionReason::BackendUnreachable (SQLSTATE 08006)."

  PoolRejectionReason:
    type: string
    x-rust-enum: true
    x-rust-derive: [Debug, Clone, Copy, PartialEq, Eq]
    enum: [backend_pool_saturated]
    description: >-
      Drives the synthesized wire ErrorResponse for a mid-session
      backend-pool-exhaustion rejection (R3, AC3), distinct from the
      existing proxy::RejectionReason (which covers frontend-admission and
      admission-handshake rejections): backend_pool_saturated maps to
      synthesized_error_response() returning an ErrorResponse with SQLSTATE
      53300 too_many_connections, message text distinguishing "backend pool
      exhausted" from the frontend-budget wording so operators can tell the
      two saturation causes apart.

  BackendPoolStats:
    type: object
    x-rust-derive: [Debug, Clone, Copy, PartialEq, Eq]
    required: [backend_active, backend_idle]
    description: "Raw counts BackendPool exposes for composition into PoolStats."
    properties:
      backend_active:
        type: integer
        x-rust-type: usize
        description: "Count of physical backend connections currently leased out (active) — includes both fresh and reused leases, both pool modes."
      backend_idle:
        type: integer
        x-rust-type: usize
        description: "Count of physical backend connections currently sitting in the shared idle set, already authenticated and liveness-eligible for reuse."

  PoolStats:
    type: object
    x-rust-derive: [Debug, Clone, Copy, PartialEq, Eq]
    required: [frontend_active, backend_active, backend_idle]
    description: >-
      The plain Rust stats API (R4) a later admin-plane WI surfaces over
      HTTP (out of scope here — this slice ships only the Rust API);
      composes the existing server_core::ConnectionBudget::active()
      (frontend admission, unchanged from WI #1288) with BackendPoolStats
      (this TD) into one snapshot.
    properties:
      frontend_active:
        type: integer
        x-rust-type: usize
        description: "ConnectionBudget::active() for the frontend listener — count of currently-admitted client connections, both pool modes."
      backend_active:
        type: integer
        x-rust-type: usize
        description: "Equal to BackendPoolStats.backend_active at snapshot time."
      backend_idle:
        type: integer
        x-rust-type: usize
        description: "Equal to BackendPoolStats.backend_idle at snapshot time."

  TransactionProxyConfig:
    type: object
    x-rust-derive: [Debug, Clone]
    required: [frontend_budget, backend_pool, wire, drain_timeout]
    description: >-
      Everything a TransactionHandler needs: reuses the same
      frontend_budget ConnectionBudget concept as SessionProxyConfig
      (frontend admission is a pool-mode-independent concern), the
      BackendPool this handler leases from, and the wire/drain bounds
      transaction-mode's own admission-handshake and per-transaction relay
      helpers need.
    properties:
      frontend_budget:
        x-rust-type: server_core::ConnectionBudget
        description: "Same admission primitive as SessionProxyConfig::frontend_budget — one shared frontend-connection cap regardless of pool mode."
      backend_pool:
        x-rust-type: crate::pool::BackendPool
        description: "The shared, capacity-bounded backend pool this handler's admission handshakes and per-transaction leases draw from."
      wire:
        x-rust-type: crate::wire::WireCodecConfig
        description: "Frame bounds used by the admission-handshake relay and the per-transaction bidirectional relay."
      drain_timeout:
        type: string
        x-rust-type: std::time::Duration
        description: "Bounds how long an in-flight admission handshake or transaction lease is allowed to keep running after DrainSignal flips to Draining before the task is abandoned; mirrors SessionProxyConfig::drain_timeout."

  TransactionHandler:
    type: object
    x-rust-derive: [Debug, Clone]
    required: [config]
    description: >-
      The tcp_server::TcpHandler impl pgpool serve binds to its listener in
      transaction mode: dispatches each accepted client through the
      admission-handshake-then-per-transaction-lease pipeline described in
      the Logic section, using its TransactionProxyConfig.
    properties:
      config:
        $ref: "#/definitions/TransactionProxyConfig"
        description: "Private field, constructed via TransactionHandler::new(config); mirrors SessionHandler's shape."

  PoolHandler:
    type: object
    x-rust-enum: true
    x-rust-derive: [Debug, Clone]
    description: >-
      TcpHandler dispatch wrapper selected once at process start from
      RuntimePlan::PoolMode: wraps the unchanged crate::proxy::SessionHandler
      for PoolMode::Session, or the new TransactionHandler for
      PoolMode::Transaction. pgpool serve constructs exactly one variant and
      binds it to the single tcp-server listener — the mode is fixed for the
      process, not renegotiated per connection.
    oneOf:
      - title: Session
        x-rust-type: crate::proxy::SessionHandler
      - title: Transaction
        $ref: "#/definitions/TransactionHandler"
```

## Config
<!-- type: config lang: yaml -->

```yaml
# BackendPool config — capacity, timeouts, and reset/liveness defaults for
# both pool modes. Reuses RuntimePlan::max_backend_connections and the
# session-mode proxy's existing backend_host/backend_port/backend_connect_timeout_ms
# (see session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#config)
# rather than re-declaring them; this section adds only the pool-specific seam.

max_backend_connections:
  source: "RuntimePlan::max_backend_connections"
  default: 512   # capacity bound shared by idle+active backend connections, both pool modes (R1)

pool_acquire_timeout_ms:
  env: PGPOOL_POOL_ACQUIRE_TIMEOUT_MS
  flag: --pool-acquire-timeout-ms
  default: 5000   # 5s; BackendPool::acquire() waits this long for an idle/freed slot before PoolError::Saturated (R3, AC3)

pool_liveness_check:
  source: "always-on, no config seam in this slice"
  default: enabled   # non-blocking peek read on every idle-pop before handing a connection out (R1); a dead peer is dropped and the acquire loop retries

pool_reset_statement:
  source: "hard-coded, no config seam in this slice"
  default: "DISCARD ALL"   # sent by pgpool itself (acting as its own client toward the backend) before LeaseDisposition::ReturnToIdle (R1, AC2); never sent on the Close disposition

pool_mode:
  source: "RuntimePlan::pool_mode"
  default: transaction   # PoolMode::Transaction is RuntimePlan::default(); PoolMode::Session reuses WI #1288's SessionHandler unchanged, now capacity-bounded through this same BackendPool

# Session mode continues to source its backend endpoint / backend_connect_timeout_ms
# from SessionProxyConfig (see the session-mode proxy TD's Config section);
# transaction mode's admission-handshake backend endpoint is the SAME
# BackendEndpointConfig — no separate per-mode backend target in this slice,
# and no multiple-backend-database/user keying (adapter-boundary epic #1283).
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: apps-pgpool-backend-pool-verification
requirements:
  ac1a_transaction_mode_reuses_backend_connections:
    id: AC1
    text: "Across N sequential client transactions in transaction mode, the number of distinct physical backend connections opened is far fewer than the client/transaction count (idle reuse via BackendPool::acquire() is exercised, not a fresh connect per transaction)."
    kind: integration
    risk: high
    verify: "pool_modes::transaction_mode_reuses_backend_connections_across_sequential_transactions"
  ac1b_concurrent_transactions_isolated_on_distinct_backends:
    id: AC1
    text: "Concurrent transactions from different clients are correctly isolated: each holds its own distinct leased backend connection at the same time, with no frame cross-talk between them."
    kind: integration
    risk: high
    verify: "pool_modes::concurrent_transactions_isolated_on_distinct_backends"
  ac2_no_session_state_leak_across_leases:
    id: AC2
    text: "A SET/temp-table fixture proves no session-state leak across transaction-mode leases: a value set (or temp table created) by one transaction on a backend is not observable by the next transaction that reuses the same backend connection, because DISCARD ALL ran between owners."
    kind: regression
    risk: high
    verify: "pool_modes::reset_between_owners_prevents_session_state_leak_across_transaction_leases"
  ac3a_saturation_wait_then_acquire_succeeds:
    id: AC3
    text: "When the backend pool is saturated, a waiting client that is unblocked by another lease's release within acquire_timeout successfully acquires a backend and proceeds — it is never silently dropped."
    kind: integration
    risk: medium
    verify: "pool_modes::saturation_wait_then_acquire_succeeds_when_lease_frees"
  ac3b_saturation_timeout_produces_typed_error:
    id: AC3
    text: "When the backend pool stays saturated past acquire_timeout, the waiting client receives a typed wire ErrorResponse (SQLSTATE 53300, PoolRejectionReason::BackendPoolSaturated) and its socket is closed cleanly — it never hangs indefinitely."
    kind: integration
    risk: high
    verify: "pool_modes::saturation_timeout_produces_typed_error_response"
  ac4_churn_100_cycles_holds_backend_count_stable:
    id: AC4
    text: "A churn test running 100+ acquire/release cycles (mixed ReturnToIdle and Close dispositions, mixed session/transaction activity) holds the backend connection count stable throughout, with no leaked connection or file descriptor at the end."
    kind: regression
    risk: high
    verify: "pool_modes::churn_100_cycles_holds_backend_count_stable_no_leak"
  ac5_stats_api_matches_fixture_expectations:
    id: AC5
    text: "PoolStats (frontend_active, backend_active, backend_idle) reports counts matching fixture expectations at each phase of a scripted sequence: before any client connects, after admission, after a transaction lease is acquired, and after it is released back to idle."
    kind: functional
    risk: medium
    verify: "pool_modes::stats_api_reports_expected_counts_at_each_phase"
  r1a_acquire_reuses_idle_after_liveness_pass:
    id: R1
    text: "BackendPool::acquire() returns an existing idle connection (not a fresh connect) when the idle set is non-empty and the popped connection's non-blocking liveness peek succeeds."
    kind: functional
    risk: medium
    verify: "pool::acquire_reuses_idle_connection_after_liveness_check_passes"
  r1b_acquire_drops_dead_idle_and_retries:
    id: R1
    text: "BackendPool::acquire() drops an idle connection whose liveness peek indicates the peer is gone, frees its capacity slot, and continues the acquire attempt (fresh-connect or wait) rather than handing back a dead connection."
    kind: functional
    risk: medium
    verify: "pool::acquire_drops_dead_idle_connection_and_retries"
  r1c_reset_sent_before_return_to_idle:
    id: R1
    text: "BackendPool::release(id, stream, LeaseDisposition::ReturnToIdle) sends DISCARD ALL and awaits its ReadyForQuery before the connection is added to the idle set."
    kind: functional
    risk: high
    verify: "pool::release_return_to_idle_sends_discard_all_before_reuse"
  r1d_reset_failure_closes_instead_of_reuse:
    id: R1
    text: "When the DISCARD ALL reset itself fails (backend EOF/error/timeout during reset), release() closes the connection and frees its capacity slot instead of adding it to the idle set — a failed reset never yields a reused connection."
    kind: regression
    risk: high
    verify: "pool::release_return_to_idle_closes_connection_when_reset_fails"
  r2a_transaction_lease_boundaries_track_ready_for_query:
    id: R2
    text: "In transaction mode, a backend lease is acquired on the first frontend frame after the client is idle-with-no-lease, and released back through LeaseDisposition::ReturnToIdle exactly when the leased backend's ReadyForQuery reports TransactionStatus::Idle."
    kind: functional
    risk: high
    verify: "pool::transaction_lease_acquired_on_first_frame_and_released_on_ready_for_query_idle"
  r2b_session_mode_holds_one_lease_for_whole_session:
    id: R2
    text: "In session mode, exactly one backend lease is held for the entire session lifetime (acquire_fresh() at connect, release(..., Close) at teardown) — session mode's per-message relay/auth behavior is otherwise unchanged from WI #1288."
    kind: regression
    risk: medium
    verify: "pool::session_mode_lease_held_for_whole_session_unchanged_from_1288"
  r3a_acquire_waits_then_succeeds:
    id: R3
    text: "BackendPool::acquire() blocks a caller when the pool is at max_backend_connections with an empty idle set, and returns a lease as soon as another holder's release() frees a slot, provided this happens within acquire_timeout."
    kind: functional
    risk: medium
    verify: "pool::acquire_waits_for_release_when_saturated_then_succeeds"
  r3b_acquire_times_out_with_saturated_error:
    id: R3
    text: "BackendPool::acquire() returns PoolError::Saturated after waiting acquire_timeout with no slot freed, rather than blocking indefinitely."
    kind: functional
    risk: high
    verify: "pool::acquire_times_out_with_saturated_error_after_acquire_timeout"
  r3c_saturated_error_maps_to_typed_response:
    id: R3
    text: "PoolError::Saturated maps to PoolRejectionReason::BackendPoolSaturated, whose synthesized_error_response() produces a wire ErrorResponse with SQLSTATE 53300."
    kind: functional
    risk: medium
    verify: "pool::saturated_pool_error_maps_to_synthesized_error_response_53300"
  r4_stats_snapshot_composes_frontend_and_backend_counts:
    id: R4
    text: "PoolStats::snapshot composes ConnectionBudget::active() (frontend_active) with BackendPool's own backend_active/backend_idle counters into one consistent snapshot."
    kind: functional
    risk: low
    verify: "pool::stats_snapshot_reports_frontend_backend_active_and_idle_counts"
  r5_dropped_lease_without_release_does_not_leak_capacity:
    id: R5
    text: "If a BackendLease is dropped without an explicit release() call (e.g. task panic/cancellation), the pool's capacity accounting still frees the slot (RAII-style guard on the lease) rather than leaking it permanently."
    kind: regression
    risk: medium
    verify: "pool::dropped_lease_without_explicit_release_does_not_leak_capacity_slot"
---
flowchart TD
    ac1a_transaction_mode_reuses_backend_connections["AC1: transaction mode reuses backend connections<br/>pool_modes::transaction_mode_reuses_backend_connections_across_sequential_transactions"]
    ac1b_concurrent_transactions_isolated_on_distinct_backends["AC1: concurrent transactions isolated<br/>pool_modes::concurrent_transactions_isolated_on_distinct_backends"]
    ac2_no_session_state_leak_across_leases["AC2: no session-state leak across leases<br/>pool_modes::reset_between_owners_prevents_session_state_leak_across_transaction_leases"]
    ac3a_saturation_wait_then_acquire_succeeds["AC3: saturation wait-then-acquire succeeds<br/>pool_modes::saturation_wait_then_acquire_succeeds_when_lease_frees"]
    ac3b_saturation_timeout_produces_typed_error["AC3: saturation timeout produces typed error<br/>pool_modes::saturation_timeout_produces_typed_error_response"]
    ac4_churn_100_cycles_holds_backend_count_stable["AC4: 100+ cycle churn, no leak<br/>pool_modes::churn_100_cycles_holds_backend_count_stable_no_leak"]
    ac5_stats_api_matches_fixture_expectations["AC5: stats API matches fixture<br/>pool_modes::stats_api_reports_expected_counts_at_each_phase"]
    r1a_acquire_reuses_idle_after_liveness_pass["R1: acquire reuses idle after liveness pass<br/>pool::acquire_reuses_idle_connection_after_liveness_check_passes"]
    r1b_acquire_drops_dead_idle_and_retries["R1: acquire drops dead idle, retries<br/>pool::acquire_drops_dead_idle_connection_and_retries"]
    r1c_reset_sent_before_return_to_idle["R1: reset sent before return-to-idle<br/>pool::release_return_to_idle_sends_discard_all_before_reuse"]
    r1d_reset_failure_closes_instead_of_reuse["R1: reset failure closes, not reuse<br/>pool::release_return_to_idle_closes_connection_when_reset_fails"]
    r2a_transaction_lease_boundaries_track_ready_for_query["R2: lease boundaries track ReadyForQuery<br/>pool::transaction_lease_acquired_on_first_frame_and_released_on_ready_for_query_idle"]
    r2b_session_mode_holds_one_lease_for_whole_session["R2: session mode holds one lease whole session<br/>pool::session_mode_lease_held_for_whole_session_unchanged_from_1288"]
    r3a_acquire_waits_then_succeeds["R3: acquire waits then succeeds<br/>pool::acquire_waits_for_release_when_saturated_then_succeeds"]
    r3b_acquire_times_out_with_saturated_error["R3: acquire times out with Saturated<br/>pool::acquire_times_out_with_saturated_error_after_acquire_timeout"]
    r3c_saturated_error_maps_to_typed_response["R3: Saturated maps to typed response<br/>pool::saturated_pool_error_maps_to_synthesized_error_response_53300"]
    r4_stats_snapshot_composes_frontend_and_backend_counts["R4: stats snapshot composes counts<br/>pool::stats_snapshot_reports_frontend_backend_active_and_idle_counts"]
    r5_dropped_lease_without_release_does_not_leak_capacity["R5: dropped lease does not leak capacity<br/>pool::dropped_lease_without_explicit_release_does_not_leak_capacity_slot"]
```
