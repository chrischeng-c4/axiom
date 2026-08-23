# transport-h2c

## Brief

`transport-h2c` provides reusable HTTP/2 cleartext transport. It includes a
simple client, a fixed pool, a managed frame-level pool, connection sizing, and
an optional HTTP/1.1 plus h2c per-connection server.

The crate never binds a listener. `server-http` and `server-tcp` own listener
admission and task supervision. TLS policy is outside this cleartext transport.

## Primary workflow

1. Use `h2c_client` for one prior-knowledge client.
2. Use `H2cPool` for a fixed round-robin set of clients.
3. Use `H2cManager` when the caller needs bounded admission, health checks,
   adaptive connections, GOAWAY handling, and explicit mutation ambiguity.
4. Enable the `server` feature only when a listener owner needs to serve one
   accepted stream as HTTP/1.1 or h2c.

## Choose a client surface

| Surface | Use it for | Current behavior |
|---|---|---|
| `h2c_client` | Simple in-cluster prior-knowledge calls | Builds one `reqwest` HTTP/2 cleartext client. |
| `H2cPool` | Fixed multi-core distribution | Sends requests round-robin over a fixed number of clients. |
| `H2cManager` | Long-lived service traffic | Uses least-loaded dispatch, bounded admission, adaptive size, liveness checks, and connection replacement. |

`recommended_h2c_connections` uses the ceiling of the natural logarithm of
target concurrency. It clamps the result between one connection and available
CPU parallelism. Callers can select an exact fixed size instead.

The manager retries a safe request once when its connection is lost. It does
not replay a dispatched mutation because the server may have applied it. That
failure is reported as an ambiguous mutation outcome.

## Serve one accepted connection

The optional `server` feature accepts an already-open stream. It detects and
serves HTTP/1.1 or prior-knowledge h2c. Lifecycle-aware forms stop new work on
drain, send the HTTP/2 shutdown signal, wait within the supplied deadline, and
return a `ConnectionReport`.

The report separates completed, refused, timed-out, and ambiguous requests. It
also records the detected protocol and terminal reason. The listener owner
aggregates those per-connection facts.

## Contract discovery

| Need | Source of truth |
|---|---|
| Public Rust API | `cargo doc -p transport-h2c --no-deps` |
| Simple client, fixed pool, and sizing | `libs/transport-h2c/src/lib.rs` |
| Managed connection pool | `libs/transport-h2c/src/manager.rs` and `conn.rs` |
| Error and ambiguity contract | `libs/transport-h2c/src/error.rs` |
| Optional per-connection server | `libs/transport-h2c/src/server.rs` |
| Executable behavior | `cargo test -p transport-h2c` |

## Capabilities

Every entry below is an equal library capability. Each source states its direct
contribution.

### Capability index

| Capability | ID | User promise | Sources |
|---|---|---|---|
| h2c client helpers | `h2c-client-helpers` | Build a prior-knowledge h2c client or a fixed round-robin pool. | `libs/transport-h2c` |
| Connection sizing | `connection-sizing` | Select a bounded connection count from target concurrency and CPU parallelism. | `libs/transport-h2c` |
| Managed h2c pool | `managed-h2c-pool` | Bound, observe, grow, shrink, and repair frame-level h2c connections. | `libs/transport-h2c` |
| Per-connection HTTP serving | `per-connection-http-serving` | Serve one accepted stream as HTTP/1.1 or h2c without taking listener ownership. | `libs/transport-h2c`, `libs/server-lifecycle` |
| Graceful connection drain | `graceful-connection-drain` | Stop admission and report completed, refused, timed-out, and ambiguous work during drain. | `libs/transport-h2c`, `libs/server-lifecycle` |

### h2c client helpers

- ID: `h2c-client-helpers`
- Promise: Build one HTTP/2 cleartext prior-knowledge client or distribute calls
  over a fixed round-robin pool.
- Sources:
  - [`libs/transport-h2c`](./) provides client construction, optional timeout
    and user agent, fixed pool creation, and GET and POST helpers.
- Gate: `cargo test -p transport-h2c`

### Connection sizing

- ID: `connection-sizing`
- Promise: Return a deterministic logarithmic connection count bounded by one
  and caller-selected or available CPU parallelism.
- Sources:
  - [`libs/transport-h2c`](./) provides the sizing formula, explicit-parallelism
    form, CPU lookup, and boundary tests.
- Gate: `cargo test -p transport-h2c`

### Managed h2c pool

- ID: `managed-h2c-pool`
- Promise: Dispatch through healthy least-loaded connections, cap in-flight
  work, adapt pool size, replace dead connections, and expose pool statistics.
- Sources:
  - [`libs/transport-h2c`](./) provides the frame-level connection driver,
    manager, health checks, sizing, timeouts, GOAWAY handling, and safe retry.
- Gate: `cargo test -p transport-h2c --test manager`

### Per-connection HTTP serving

- ID: `per-connection-http-serving`
- Promise: With the `server` feature, serve one caller-accepted stream as
  HTTP/1.1 or h2c and return its terminal report.
- Sources:
  - [`libs/transport-h2c`](./) provides protocol detection and per-connection
    Hyper serving.
  - [`libs/server-lifecycle`](../server-lifecycle/) supplies the lifecycle and
    shutdown deadline observed by lifecycle-aware forms.
- Gate: `cargo test -p transport-h2c`

### Graceful connection drain

- ID: `graceful-connection-drain`
- Promise: Close admission during drain, give active work its supplied
  deadline, and distinguish refused, timed-out, and ambiguous mutations.
- Sources:
  - [`libs/transport-h2c`](./) provides request accounting, HTTP/2 shutdown,
    deadline handling, mutation classification, and `ConnectionReport`.
  - [`libs/server-lifecycle`](../server-lifecycle/) supplies the lifecycle
    observation and absolute deadline.
- Gate: `cargo test -p transport-h2c --test graceful_drain`

## Supporting documents

| Document | Use it for |
|---|---|
| [STATUS.md](STATUS.md) | Current transport support and limits |
| [ROADMAP.md](ROADMAP.md) | Explicit transport non-goals |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Edit rules and required verification |
