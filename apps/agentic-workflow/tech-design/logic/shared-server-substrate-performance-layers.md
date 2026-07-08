---
id: shared-server-substrate-performance-layers
summary: Add layered shared server substrate crates for efficient TCP and HTTP runtimes below the service archetype.
fill_sections: [logic, unit-test]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: shared-service-kit-substrate
    claim: shared-service-kit-substrate
    coverage: partial
    rationale: "Extends the shared service kit convention with reusable server runtime layers so apps and service archetype adopters do not hand-roll accept loops, drain behavior, or connection budgeting."
---

# TD: shared server substrate performance layers

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shared-server-substrate-runtime-layers
entry: app_runtime
nodes:
  app_runtime: { kind: start, label: "Apps and service runtimes" }
  service_http: { kind: process, label: "libs/service-http service archetype shell" }
  http_server: { kind: process, label: "libs/http-server HTTP runtime" }
  tcp_server: { kind: process, label: "libs/tcp-server TCP accept/runtime" }
  server_core: { kind: process, label: "libs/server-core lifecycle and budgets" }
  h2c: { kind: process, label: "libs/h2c HTTP/1.1 + h2c transport" }
  bind: { kind: terminal, label: "bind config" }
  drain: { kind: terminal, label: "shutdown and drain signal" }
  budget: { kind: terminal, label: "connection budget" }
  socket: { kind: terminal, label: "socket options + TCP_NODELAY" }
  tasks: { kind: terminal, label: "per-connection JoinSet supervision" }
  streams: { kind: terminal, label: "tunable max concurrent streams" }
edges:
  - { from: app_runtime, to: service_http }
  - { from: app_runtime, to: http_server }
  - { from: service_http, to: http_server }
  - { from: http_server, to: tcp_server }
  - { from: http_server, to: server_core }
  - { from: http_server, to: h2c }
  - { from: tcp_server, to: server_core }
  - { from: server_core, to: bind }
  - { from: server_core, to: drain }
  - { from: server_core, to: budget }
  - { from: tcp_server, to: socket }
  - { from: tcp_server, to: tasks }
  - { from: h2c, to: streams }
---
flowchart TD
    AppRuntime[Apps and service runtimes]
    ServiceHttp[libs/service-http\nservice archetype shell]
    HttpServer[libs/http-server\nHTTP runtime]
    TcpServer[libs/tcp-server\nTCP accept/runtime]
    ServerCore[libs/server-core\nlifecycle and budgets]
    H2c[libs/h2c\nHTTP/1.1 + h2c transport]

    AppRuntime --> ServiceHttp
    AppRuntime --> HttpServer
    ServiceHttp --> HttpServer
    HttpServer --> TcpServer
    HttpServer --> ServerCore
    HttpServer --> H2c
    TcpServer --> ServerCore

    ServerCore --> Bind[bind config]
    ServerCore --> Drain[shutdown and drain signal]
    ServerCore --> Budget[connection budget]
    TcpServer --> Socket[socket options + TCP_NODELAY]
    TcpServer --> Tasks[per-connection JoinSet supervision]
    H2c --> Streams[tunable max concurrent streams]
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: shared-server-substrate-performance-layers-verification
requirements:
  focused_workspace_gates:
    id: R4
    text: "The focused substrate packages format and compile together through the workspace dependency graph."
    kind: regression
    risk: medium
    verify: cargo fmt -p server-core -p tcp-server -p http-server -p h2c -p service-http --check && cargo check -p server-core -p tcp-server -p http-server -p h2c -p service-http
  http_h2c_options:
    id: R2
    text: "h2c/http-server expose tunable server options for HTTP/2 max concurrent streams and drain timeout while preserving the default serve path."
    kind: functional
    risk: medium
    verify: cargo test -p h2c -p http-server
  service_http_compatibility:
    id: R3
    text: "service-http delegates transport through http-server without changing its probe, OpenAPI, metrics, error-envelope, or tracing surface."
    kind: regression
    risk: high
    verify: cargo test -p service-http
  tcp_hot_path:
    id: R1
    text: "tcp-server exposes a zero-boxing handler path, socket bind options, TCP_NODELAY, connection admission, and bounded drain behavior."
    kind: functional
    risk: high
    verify: cargo test -p tcp-server
---
flowchart TD
    r1[R1 tcp hot path] --> cargo_test_p_tcp_server[cargo test -p tcp-server]
    r2[R2 http h2c options] --> cargo_test_p_h2c_p_http_server[cargo test -p h2c -p http-server]
    r3[R3 service http compatibility] --> cargo_test_p_service_http[cargo test -p service-http]
    r4[R4 focused workspace gates] --> cargo_fmt_p_server_core_p_tcp_server_p_http_server_p_h2c_p_service_http_check_cargo_check_p_server_core_p_tcp_server_p_http_server_p_h2c_p_service_http[cargo fmt -p server-core -p tcp-server -p http-server -p h2c -p service-http --check && cargo check -p server-core -p tcp-server -p http-server -p h2c -p service-http]
```
