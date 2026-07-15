---
id: '1776'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shared-http-runtime-lifecycle-ownership
entry: start
nodes:
  start: { kind: start, label: "HTTP runtime request" }
  lifecycle: { kind: process, label: "server-lifecycle owns bind, drain/readiness, shutdown, budgets, and connection metric events" }
  tcp: { kind: process, label: "server-tcp owns listener admission, per-connection supervision, metric callbacks, and bounded drain" }
  http: { kind: process, label: "server-http owns the only HTTP listener facade and maps each accepted TCP stream to HTTP/1.1+h2c" }
  transport: { kind: process, label: "transport-h2c owns outbound clients/pools plus per-connection h2c protocol machinery, never a listener loop" }
  service: { kind: process, label: "service-http owns probes, OpenAPI/docs, HTTP errors, and request-policy adapters" }
  apps: { kind: terminal, label: "Lumen, Tape, Keep, Relay, Courier, and Pgpool compose one lifecycle contract without route changes" }
  shutdown: { kind: terminal, label: "One shutdown signal flips shared drain state, stops admission, and drains supervised connections" }
edges:
  - { from: start, to: lifecycle }
  - { from: lifecycle, to: tcp }
  - { from: tcp, to: http }
  - { from: http, to: transport }
  - { from: http, to: service }
  - { from: service, to: apps }
  - { from: lifecycle, to: shutdown }
  - { from: tcp, to: shutdown }
---
flowchart TD
  start([HTTP runtime request]) --> lifecycle[server-lifecycle: bind, drain/readiness, shutdown, budgets, metric events]
  lifecycle --> tcp[server-tcp: accept, admission, supervision, metric callbacks, bounded drain]
  tcp --> http[server-http: sole listener facade and HTTP connection dispatch]
  http --> transport[transport-h2c: outbound pools plus per-connection HTTP/1.1+h2c protocol]
  http --> service[service-http: probes, OpenAPI/docs, errors, request policy]
  service --> apps([service and tool consumers preserve public routes])
  lifecycle --> shutdown([shared drain state stops admission])
  tcp --> shutdown
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/server-lifecycle/src/readiness.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: Own the protocol-neutral readiness/draining observation contract.
  - path: libs/server-lifecycle/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Export the single readiness and shutdown lifecycle surface.
  - path: libs/server-tcp/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Invoke connection metric callbacks around admission, rejection, and completion.
  - path: libs/transport-h2c/src/server.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Narrow server support to one accepted connection without listener ownership.
  - path: libs/transport-h2c/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Export per-connection protocol machinery and remove listener-level serve exports.
  - path: libs/transport-h2c/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Align optional dependencies and crate description with the narrowed transport boundary.
  - path: libs/transport-h2c/src/llm.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Advertise the source-backed transport boundary without listener ownership.
  - path: libs/server-http/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Own the HTTP listener facade by composing server-tcp with per-connection h2c.
  - path: libs/server-http/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Own HTTP runtime dependencies and expose lifecycle-aware configuration.
  - path: libs/server-http/README.md
    action: create
    section: logic
    impl_mode: hand-written
    description: Define the shared HTTP runtime capability and ownership boundary.
  - path: libs/service-http/src/readiness.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Adapt the server-lifecycle readiness contract to HTTP probes without redefining it.
  - path: libs/service-http/src/signal.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Delegate shutdown behavior to server-lifecycle without a parallel implementation.
  - path: libs/service-http/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Compose the protocol-neutral lifecycle owner explicitly.
  - path: libs/service-http/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Limit service-http ownership to the service HTTP policy shell.
  - path: README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Correct the shared library inventory ownership descriptions.
  - path: CONTRIBUTING.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Record the listener, lifecycle, and per-connection transport boundaries.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: shared-http-runtime-lifecycle-convergence-verification
requirements:
  connection_metric_events:
    id: R2
    text: "server-tcp emits accepted, rejected, and closed metric callbacks with permits and task completion preserving current admission behavior."
    kind: functional
    risk: high
    verify: cargo test -p server-tcp
  service_consumer_regression:
    id: R4
    text: "Lumen, Tape, Keep, Relay, Courier, and Pgpool preserve their existing HTTP and drain contracts after the shared runtime convergence."
    kind: regression
    risk: high
    verify: cargo test -p lumen --features operator --test operator_render; cargo test -p tape --test http_transport; cargo test -p keep; cargo test -p relay --test http2_transport; cargo test -p courier; cargo test -p pgpool --test admin_plane
  single_readiness_shutdown_contract:
    id: R3
    text: "server-lifecycle owns readiness and signal/drain behavior while service-http only adapts that contract to probe routes."
    kind: regression
    risk: high
    verify: cargo test -p server-lifecycle -p service-http
  sole_http_listener_owner:
    id: R1
    text: "server-http owns listener admission and dispatches accepted streams through server-tcp into per-connection HTTP/1.1+h2c handling; transport-h2c exposes no listener-level serve API."
    kind: functional
    risk: high
    verify: cargo test -p server-http -p transport-h2c
---
flowchart TD
    r1[R1 sole http listener owner] --> cargo_test_p_server_http_p_transport_h2c[cargo test -p server-http -p transport-h2c]
    r2[R2 connection metric events] --> cargo_test_p_server_tcp[cargo test -p server-tcp]
    r3[R3 single readiness shutdown contract] --> cargo_test_p_server_lifecycle_p_service_http[cargo test -p server-lifecycle -p service-http]
    r4[R4 service consumer regression] --> cargo_test_p_lumen_features_operator_test_operator_render_cargo_test_p_tape_test_http_transport_cargo_test_p_keep_cargo_test_p_relay_test_http2_transport_cargo_test_p_courier_cargo_test_p_pgpool_test_admin_plane[cargo test -p lumen --features operator --test operator_render; cargo test -p tape --test http_transport; cargo test -p keep; cargo test -p relay --test http2_transport; cargo test -p courier; cargo test -p pgpool --test admin_plane]
```
