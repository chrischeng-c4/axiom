# raft-runtime

## Brief

`raft-runtime` drives `raft-core` for caller-supplied state machines over plain
h2c or mutually authenticated HTTP/2 peer transport, with snapshot, compaction,
and read-your-write propose support.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Shared Raft Host Driver | - | h2c peer host for raft-core state machines |
| Shared Peer mTLS Transport | #1643 | identity-validated HTTP/2 peers with atomic certificate reload |
| Committed Executor Fencing | #1854 | deterministic owner/epoch/expiry transitions embedded by service state machines |
| Durable Commit Recovery | #1854 | persisted commit watermark, immediate committed-log replay, cold snapshot seed, change-only hard-state fsync, and shared applied-index marker |

### Shared Raft Host Driver

Services can host Raft state machines through a shared h2c driver instead of
duplicating peer transport and read consistency plumbing.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `raft_runtime`.
- Gate — behavior: `cargo test -p raft-runtime` - host, config, store, and read
  consistency coverage
- Gate: `cargo test -p raft-runtime`
- Source: `libs/raft-runtime/src/lib.rs`
- Evidence: `cargo test -p raft-runtime`; libs/raft-runtime/src/lib.rs

### Shared Peer mTLS Transport

Stateful services can serve Raft routes on a dedicated mutually authenticated
HTTP/2 listener and dial peers through the same reloadable transport snapshot.
Invalid client chains, server identities, expired certificates, and malformed
reloads fail closed before Raft dispatch; existing plain h2c APIs remain
compatible.

- Root WI: #1643
- Surfaces: Rust API: `raft_runtime::PeerTransport`,
  `raft_runtime::RaftHost::spawn_with_peer_transport`,
  `raft_runtime::ClusterTopology::from_env_with_scheme`.
- Gate — security: `cargo test -p raft-runtime --test peer_mtls` - mutual
  identity, trust, expiry, and reload coverage
- Gate: `cargo test -p raft-runtime`
- Gate: `cargo test -p raft-runtime --test peer_mtls`
- Source: `libs/raft-runtime/src/peer_transport.rs`
- Evidence: `cargo test -p raft-runtime --test peer_mtls`;
  libs/raft-runtime/src/peer_transport.rs

### Committed Executor Fencing

Effectful services embed one application-neutral owner/epoch/expiry state in
their Raft state machine. No executor token exists before assignment commit;
expiry is an explicit replicated transition; reassignment increments the epoch;
and late outcomes from an earlier owner are rejected. The service continues to
own assignment keys, domain commands, capacity policy, and external effects.

- Root WI: #1854
- Surfaces: Rust API: `FencedAssignment`, `FenceToken`, `AssignmentError`.
- Gate — behavior: `cargo test -p raft-runtime --test fenced_assignment` -
  commit-before-effect, explicit expiry, reassignment, and stale-owner
  rejection
- Gate: `cargo test -p raft-runtime`
- Gate: `cargo test -p raft-runtime --test fenced_assignment`
- Source: `libs/raft-runtime/src/fenced_assignment.rs`
- Evidence: `cargo test -p raft-runtime --test fenced_assignment`;
  libs/raft-runtime/src/fenced_assignment.rs

### Durable Commit Recovery

Restarting a replica restores the committed watermark and applies every durable
committed entry before accepting fresh proposals. A cold bootstrap may seed an
empty store with an exact state-machine snapshot, while services with their own
durable data plane can share the small fsynced applied-index floor instead of
forking marker logic. `RaftStore` skips byte-identical hard-state writes so
idle ticks do not create avoidable fsync pressure.

- Root WI: #1854
- Surfaces: Rust API: `RaftStore::seed_snapshot`, `AppliedIndexStore`;
  persisted `raft_core::PersistedState::commit_index`.
- Gate — stability: `cargo test -p raft-core -p raft-runtime` - committed
  entries replay before new proposals, snapshot seed refuses overwrite, and
  unchanged ticks do not rewrite hard state
- Gate: `cargo test -p raft-core -p raft-runtime`
- Source: `libs/raft-runtime/src/store.rs`,
  `libs/raft-runtime/src/applied_index_store.rs`
- Evidence: raft-core/runtime restart, seed, and store tests
