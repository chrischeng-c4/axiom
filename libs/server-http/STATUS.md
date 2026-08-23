# server-http status

## Scope

This document describes the current source contract for the reusable
`server-http` crate. It does not claim that the named gate ran in this working
session.

Use the [README](README.md) for the listener workflow. Use the
[roadmap](ROADMAP.md) for explicit non-goals.

## State definitions

| State | Meaning |
|---|---|
| Supported | The current source has a public contract, an implementation, and a named executable gate for the stated scope. |
| Limited | The current source supports the stated scope, but the Limits cell names a material boundary. |
| Not supported | The behavior is not part of the current product contract. The Evidence cell points to a future outcome or a non-goal. |

## Support matrix

| Surface | ID | State | Supported scope | Limits | Evidence |
|---|---|---|---|---|---|
| HTTP/1.1 and h2c listener | `http1-h2c-listener` | Supported | One supplied TCP listener can serve one Axum router as HTTP/1.1 and prior-knowledge h2c. | The caller binds the listener and supplies the routes. | `cargo test -p server-http --test ownership` |
| Lifecycle drain and terminal report | `lifecycle-drain-terminal-report` | Supported | One caller lifecycle controls accept and stream drain, and serving returns connection and request-stream terminal counts. | The caller decides when to enter drain and owns domain readiness policy. | `cargo test -p server-http --test lifecycle_composition` |
| TLS serving | `tls-serving` | Supported | The listener can terminate TLS from a caller-supplied rustls configuration and refuse service when no valid configuration is active. | The caller supplies certificate material, validation, ALPN, and identity policy. | `cargo test -p server-http --test tls_reload` |
| Accept-time TLS configuration reload | `accept-time-tls-config-reload` | Supported | The listener reads `ServerConfigSource` once per accept, so a new connection uses newly active configuration without a listener rebind. | Existing connections keep the configuration selected at their accept. | `cargo test -p server-http --test tls_reload` |

## Evidence policy

The commands above are required gates for each supported scope. This document
does not store execution output. CI and local test logs own run evidence.

Update a row with any public API, behavior, or evidence change.
