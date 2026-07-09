---
id: apps-pgpool-session-proxy
summary: pgpool `serve` entrypoint and session-mode (1:1) proxy - a tcp-server TcpHandler-backed frontend listener with ConnectionBudget admission and a wire-level ErrorResponse rejection when saturated, a per-client backend TCP connection with frame-aware auth passthrough (cleartext/MD5/SCRAM relayed via decode+re-encode of the wire codec's typed messages, credentials never persisted), bidirectional relay until Terminate/EOF, and server-core DrainController-driven graceful shutdown bounded by a drain timeout.
capability_refs:
  - id: postgres-pooler-core
    role: primary
    gap: serve-entrypoint-and-drain
    claim: serve-entrypoint-and-drain
    coverage: full
    rationale: "Defines and closes the serve-entrypoint-and-drain work root: the pgpool serve CLI verb, tcp-server-backed frontend listener with ConnectionBudget admission, per-client 1:1 backend session proxy with auth passthrough, and drain-bounded graceful shutdown, verified by cargo test -p pgpool --test session_proxy against a real Postgres backend (graceful-skip) plus offline unit coverage."
  - id: cli-interface
    role: contributes
    gap: serve-by-default-entrypoint
    claim: serve-by-default-entrypoint
    coverage: full
    rationale: "Adds the pgpool serve subcommand to the compiled CLI (cargo test -p pgpool --test cli_contract) and to the llm workflow topic, closing the serve-by-default-entrypoint work root's remaining gap: the process entrypoint that actually admits clients."
fill_sections: [logic, state-machine, schema, config, unit-test]
---

# pgpool session-mode proxy — serve entrypoint, auth passthrough, drain

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-session-proxy-logic-flow
entry: cli_serve_entry
nodes:
  cli_serve_entry:
    kind: start
    label: "`pgpool serve` builds TcpServerConfig from RuntimePlan (bind, TcpSocketOptions, drain_timeout) with NO tcp-server-level ConnectionBudget wired in — admission moves into SessionHandler so a rejection can still write a wire frame — then calls tcp_server::bind + tcp_server::serve(listener, config, SessionHandler, server_core::signal::wait_shutdown_signal())"
  tcp_accept:
    kind: process
    label: "tcp-server's serve_arc accepts a raw TCP connection and invokes SessionHandler::handle(stream, ConnectionContext) for every accepted socket"
  admission_check:
    kind: decision
    label: "SessionHandler's own ConnectionBudget::try_acquire() (RuntimePlan::frontend_budget) succeeds for this connection (R1)?"
  reject_saturated:
    kind: terminal
    label: "Saturated: encode BackendMessage::ErrorResponse (SQLSTATE 53300 too_many_connections) and write it directly to the client stream, then close the socket without touching the backend or any other session (AC3)"
  connect_backend:
    kind: process
    label: "Admitted, permit held for the session lifetime: TCP-connect to the configured backend endpoint (PGPOOL_BACKEND_ADDR/--backend-addr host:port, bounded by PGPOOL_BACKEND_CONNECT_TIMEOUT_MS/--backend-connect-timeout-ms) (R3)"
  backend_connect_ok:
    kind: decision
    label: "Backend TCP connect succeeded within the connect timeout?"
  reject_backend_unreachable:
    kind: terminal
    label: "Unreachable/timed out: encode BackendMessage::ErrorResponse (SQLSTATE 08006 connection_failure) to the client, release the ConnectionBudget permit, close the client socket"
  relay_startup:
    kind: process
    label: "Frame-aware startup relay: a frontend-role FrameReader fed from the client stream decodes the client's untagged StartupMessage (or SSLRequest, rejected — TLS is out of scope), which is re-encoded and forwarded byte-identically to the backend stream (R2)"
  relay_auth:
    kind: process
    label: "Auth passthrough loop: alternately decode backend-role BackendMessage frames from the backend stream and frontend-role FrontendMessage frames from the client stream, re-encoding and forwarding every Authentication*/PasswordMessage/SaslInitialResponse/SaslResponse frame verbatim in both directions; pgpool treats password/SASL payload bytes as opaque relay data only and never persists them (R2, AC2)"
  auth_result:
    kind: decision
    label: "Backend emits AuthenticationOk before any ErrorResponse?"
  relay_error_from_backend:
    kind: terminal
    label: "Backend emits ErrorResponse during startup/auth (bad credentials, SCRAM failure, ...): forward it to the client verbatim, release the permit, close both sides (no retry, no credential caching)"
  relay_ready:
    kind: process
    label: "Forward remaining backend startup messages (ParameterStatus*, BackendKeyData, ReadyForQuery) to the client; the backend-role FrameReader's transaction_status() records the initial TransactionStatus"
  bidi_relay:
    kind: process
    label: "Bidirectional relay: two concurrent tasks each decode WireMessage frames from one leg (frontend FrameReader on client->backend, backend FrameReader on backend->client) and re-encode+forward them to the other leg, until a frontend Terminate message, clean EOF, or a FrameError ends that leg (R2)"
  relay_end:
    kind: decision
    label: "Which condition ended the bidirectional relay?"
  client_terminate:
    kind: terminal
    label: "Client sent Terminate (or closed cleanly): forward Terminate to the backend if not already sent, close the backend connection, release the ConnectionBudget permit — a clean session end"
  backend_closed_or_error:
    kind: terminal
    label: "Backend closed the connection, or a leg hit FrameError (malformed/oversized frame): close the client socket, release the permit; a FrameError never forwards the offending bytes, it only ends that leg"
  drain_interaction:
    kind: process
    label: "Concurrently, DrainSignal (from ConnectionContext.drain) flips to Draining when the process receives SIGTERM/SIGINT: tcp-server's accept loop stops taking new connections immediately, while this session's bidi_relay keeps running unaffected until the client/backend end it or tcp_server's TcpServerConfig.drain_timeout elapses, at which point the task is abandoned (R4, AC4)"
edges:
  - from: cli_serve_entry
    to: tcp_accept
    label: "listener bound, serve loop running"
  - from: tcp_accept
    to: admission_check
    label: "connection accepted"
  - from: admission_check
    to: reject_saturated
    label: "budget exhausted"
  - from: admission_check
    to: connect_backend
    label: "permit acquired"
  - from: connect_backend
    to: backend_connect_ok
    label: "connect attempted"
  - from: backend_connect_ok
    to: reject_backend_unreachable
    label: "connect failed/timed out"
  - from: backend_connect_ok
    to: relay_startup
    label: "backend socket established"
  - from: relay_startup
    to: relay_auth
    label: "StartupMessage forwarded"
  - from: relay_auth
    to: auth_result
    label: "backend responds"
  - from: auth_result
    to: relay_error_from_backend
    label: "ErrorResponse (auth failed)"
  - from: auth_result
    to: relay_ready
    label: "AuthenticationOk"
  - from: relay_ready
    to: bidi_relay
    label: "ReadyForQuery forwarded to client"
  - from: bidi_relay
    to: relay_end
    label: "a leg ends"
  - from: relay_end
    to: client_terminate
    label: "Terminate or clean client EOF"
  - from: relay_end
    to: backend_closed_or_error
    label: "backend EOF or FrameError"
  - from: connect_backend
    to: drain_interaction
    label: "session task holds a DrainSignal for its whole lifetime"
---
flowchart TD
    cli_serve_entry([pgpool serve builds TcpServerConfig + SessionHandler, calls tcp_server::serve]) --> tcp_accept[serve_arc accepts a connection, calls SessionHandler::handle]
    tcp_accept --> admission_check{ConnectionBudget::try_acquire succeeds?}
    admission_check -->|budget exhausted| reject_saturated([Write ErrorResponse 53300, close socket])
    admission_check -->|permit acquired| connect_backend[TCP-connect to configured backend endpoint]
    connect_backend --> backend_connect_ok{Backend connect ok within timeout?}
    backend_connect_ok -->|failed/timed out| reject_backend_unreachable([Write ErrorResponse 08006, release permit, close])
    backend_connect_ok -->|established| relay_startup[Relay StartupMessage frame-aware]
    relay_startup --> relay_auth[Auth passthrough loop: relay Authentication*/Password/SASL frames]
    relay_auth --> auth_result{Backend emits AuthenticationOk?}
    auth_result -->|ErrorResponse| relay_error_from_backend([Forward ErrorResponse, release permit, close both sides])
    auth_result -->|AuthenticationOk| relay_ready[Forward ParameterStatus/BackendKeyData/ReadyForQuery]
    relay_ready --> bidi_relay[Bidirectional relay until Terminate/EOF/FrameError]
    bidi_relay --> relay_end{Which side ended the session?}
    relay_end -->|Terminate/clean client EOF| client_terminate([Forward Terminate, close backend, release permit])
    relay_end -->|backend EOF/FrameError| backend_closed_or_error([Close client socket, release permit])
    connect_backend -.-> drain_interaction[DrainSignal: accept loop stops on drain, in-flight relay keeps running until drain_timeout]
```
