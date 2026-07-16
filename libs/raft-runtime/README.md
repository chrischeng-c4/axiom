# raft-runtime

## Brief

`raft-runtime` drives `raft-core` for caller-supplied state machines over plain
h2c or mutually authenticated HTTP/2 peer transport, with snapshot, compaction,
and read-your-write propose support.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Raft Host Driver | - | implemented | verified | smoke | ready | h2c peer host for raft-core state machines |
| Shared Peer mTLS Transport | #1643 | implemented | verified | conformance | ready | identity-validated HTTP/2 peers with atomic certificate reload |

### Shared Raft Host Driver

ID: shared-raft-runtime-driver
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `raft_runtime`.
EC Dimensions: behavior: `cargo test -p raft-runtime` - host, config, store, and read consistency coverage
Required Verification: smoke
Promise:
Services can host Raft state machines through a shared h2c driver instead of
duplicating peer transport and read consistency plumbing.
Gate Inventory: `cargo test -p raft-runtime`; libs/raft-runtime/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-raft-runtime-driver-contract | epic | - | implemented | verified | smoke | `cargo test -p raft-runtime`; libs/raft-runtime/src/lib.rs |

### Shared Peer mTLS Transport

ID: shared-peer-mtls-transport
Type: Security
Root WI: 1643
Status: verified
Surfaces: Rust API: `raft_runtime::PeerTransport`, `raft_runtime::RaftHost::spawn_with_peer_transport`, `raft_runtime::ClusterTopology::from_env_with_scheme`.
EC Dimensions: security: `cargo test -p raft-runtime --test peer_mtls` - mutual identity, trust, expiry, and reload coverage
Required Verification: conformance
Promise:
Stateful services can serve Raft routes on a dedicated mutually authenticated
HTTP/2 listener and dial peers through the same reloadable transport snapshot.
Invalid client chains, server identities, expired certificates, and malformed
reloads fail closed before Raft dispatch; existing plain h2c APIs remain
compatible.
Gate Inventory: `cargo test -p raft-runtime`; `cargo test -p raft-runtime --test peer_mtls`; libs/raft-runtime/src/peer_transport.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-peer-mtls-transport | change | #1643 | implemented | verified | conformance | `cargo test -p raft-runtime --test peer_mtls`; libs/raft-runtime/src/peer_transport.rs |
