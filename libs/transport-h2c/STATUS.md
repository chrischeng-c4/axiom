# transport-h2c status

## Scope

This document describes the current source contract for the reusable
`transport-h2c` crate. It does not claim that the named gate ran in this working
session.

Use the [README](README.md) for the transport workflow. Use the
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
| h2c client helpers | `h2c-client-helpers` | Supported | Callers can build a prior-knowledge `reqwest` h2c client or a fixed round-robin pool. | This is cleartext HTTP/2. The fixed pool does not inspect connection health or load. | `cargo test -p transport-h2c` |
| Connection sizing | `connection-sizing` | Supported | The logarithmic sizing helper selects a connection count bounded by one and caller-selected or available CPU parallelism. | The formula is a transport recommendation. A caller can choose an exact pool size. | `cargo test -p transport-h2c` |
| Managed h2c pool | `managed-h2c-pool` | Supported | `H2cManager` provides bounded admission, least-loaded dispatch, adaptive size, liveness checks, dead-connection replacement, and statistics. | Safe requests can retry once after connection loss. Dispatched mutations report an ambiguous outcome instead of replaying. | `cargo test -p transport-h2c --test manager` |
| Per-connection HTTP serving | `per-connection-http-serving` | Supported | The optional `server` feature serves one accepted stream as HTTP/1.1 or prior-knowledge h2c. | The caller owns listener bind, accept admission, task supervision, and aggregation. | `cargo test -p transport-h2c` |
| Graceful connection drain | `graceful-connection-drain` | Supported | Lifecycle-aware serving closes admission, initiates HTTP/2 shutdown, observes one deadline, and returns terminal request accounting. | A mutation still active when the deadline expires is reported as ambiguous. | `cargo test -p transport-h2c --test graceful_drain` |

## Evidence policy

The commands above are required gates for each supported scope. This document
does not store execution output. CI and local test logs own run evidence.

Update a row with any public API, behavior, or evidence change.
