# server-http

## Brief

`server-http` is the shared listener-level HTTP runtime. It serves HTTP/1.1 and
h2c, composes listener drain with one lifecycle, returns terminal accounting,
and can terminate TLS from a configuration selected at accept time.

`server-http` does not define routes or middleware policy. It does not parse
certificates or decide which identity is valid. Those responsibilities belong
to the app, `service-http`, and `peer-tls`.

## Primary workflow

1. Bind or obtain a `TcpListener` in the service shell.
2. Build the Axum router in the app and HTTP policy layer.
3. Set stream, connection-budget, socket, metrics, and drain options.
4. Call `serve_h2c_with_lifecycle` for the production cleartext path.
5. For TLS, supply a `ServerConfigSource` that returns the active validated
   rustls configuration for each accepted connection.

## Serve HTTP on one listener

`serve_h2c_with_lifecycle` gives `server-tcp` ownership of accept admission,
task supervision, socket policy, connection budget, and the absolute shutdown
deadline. It gives each accepted stream to `transport-h2c`, which serves
HTTP/1.1 or prior-knowledge h2c.

The returned `HttpServerReport` separates accepted, rejected, completed,
failed, timed-out, and unfinished connections. It also aggregates admitted,
completed, refused, timed-out, and ambiguous request-stream counts.

`serve_h2c` and `serve_h2c_with_options` remain legacy and development
adapters. Production lifecycle composition should pass one
`LifecycleController` to `serve_h2c_with_lifecycle`.

## Serve and reload TLS

`serve_tls` terminates TLS on the shared listener. It calls the supplied
`ServerConfigSource` once after each TCP accept. A new connection therefore
uses the configuration active at its accept. An existing connection keeps its
original configuration until it ends.

If the source returns no valid configuration, the listener refuses that
connection. It does not fall back to cleartext. The supplied rustls
configuration owns ALPN choices.

This crate accepts an already-built `rustls::ServerConfig`. It never parses
PEM, validates certificate identity, or holds the last known good material.
[`peer-tls`](../peer-tls/) owns those certificate and identity mechanisms when
an app chooses that implementation.

## Contract discovery

| Need | Source of truth |
|---|---|
| Public Rust API | `cargo doc -p server-http --no-deps` |
| Listener options, lifecycle serving, and report | `libs/server-http/src/lib.rs` |
| TLS serving and accept-time selection | `libs/server-http/src/tls.rs` |
| TCP accept and supervision | `libs/server-tcp` |
| Per-connection HTTP protocol | `libs/transport-h2c` |
| Certificate parsing and identity policy | `libs/peer-tls` |
| Executable behavior | `cargo test -p server-http` |

## Capabilities

Every entry below is an equal library capability. Each source states its direct
contribution.

### Capability index

| Capability | ID | User promise | Sources |
|---|---|---|---|
| HTTP listener runtime | `http-listener-runtime` | Serve HTTP/1.1 and h2c through one caller-supplied TCP listener. | `libs/server-http`, `libs/server-tcp`, `libs/transport-h2c` |
| Lifecycle drain and report | `lifecycle-drain-report` | Drain one HTTP listener under a caller lifecycle and return bounded terminal accounting. | `libs/server-http`, `libs/server-lifecycle`, `libs/server-tcp`, `libs/transport-h2c` |
| TLS listener serving | `tls-listener-serving` | Terminate TLS without a cleartext fallback when no valid configuration is active. | `libs/server-http`, `libs/server-tcp`, `libs/transport-h2c` |
| Accept-time TLS configuration | `accept-time-tls-configuration` | Select the active rustls configuration once for every newly accepted connection. | `libs/server-http` |

### HTTP listener runtime

- ID: `http-listener-runtime`
- Promise: Serve one Axum router as HTTP/1.1 and prior-knowledge h2c through one
  supplied listener.
- Sources:
  - [`libs/server-http`](./) composes listener options with the router and
    aggregates connection results.
  - [`libs/server-tcp`](../server-tcp/) owns accept admission, socket options,
    connection budgets, and task supervision.
  - [`libs/transport-h2c`](../transport-h2c/) detects and serves the protocol on
    each accepted stream.
- Gate: `cargo test -p server-http --test ownership`

### Lifecycle drain and report

- ID: `lifecycle-drain-report`
- Promise: Close listener and stream admission under one lifecycle deadline and
  return explicit connection and request-stream terminal counts.
- Sources:
  - [`libs/server-http`](./) maps and aggregates TCP and HTTP connection facts
    into `HttpServerReport`.
  - [`libs/server-lifecycle`](../server-lifecycle/) owns lifecycle state and the
    absolute shutdown deadline.
  - [`libs/server-tcp`](../server-tcp/) drains the accept loop and supervises
    connection tasks.
  - [`libs/transport-h2c`](../transport-h2c/) drains per-connection HTTP work and
    reports mutation ambiguity.
- Gate: `cargo test -p server-http --test lifecycle_composition`

### TLS listener serving

- ID: `tls-listener-serving`
- Promise: Terminate TLS on the shared listener and refuse a connection when no
  valid configuration is active.
- Sources:
  - [`libs/server-http`](./) performs the rustls handshake, keeps bounded edge
    counters, and prevents cleartext fallback.
  - [`libs/server-tcp`](../server-tcp/) owns listener admission and task drain.
  - [`libs/transport-h2c`](../transport-h2c/) serves HTTP over the accepted TLS
    stream.
- Gate: `cargo test -p server-http --test tls_reload`

### Accept-time TLS configuration

- ID: `accept-time-tls-configuration`
- Promise: Read the active rustls configuration once per accepted connection so
  new connections can use rotated material without rebinding the listener.
- Sources:
  - [`libs/server-http`](./) defines `ServerConfigSource`, selects it at accept,
    and leaves existing connections on their selected configuration.
- Gate: `cargo test -p server-http --test tls_reload`

## Supporting documents

| Document | Use it for |
|---|---|
| [STATUS.md](STATUS.md) | Current listener and TLS support boundaries |
| [ROADMAP.md](ROADMAP.md) | Explicit listener-runtime non-goals |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Edit rules and required verification |
