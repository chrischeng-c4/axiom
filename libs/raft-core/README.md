# raft-core

## Brief

`raft-core` is a self-contained, step-driven Raft consensus core. It is
transport- and storage-agnostic so services can supply their network,
persistence, and state machine layers.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Step-Driven Raft Consensus Core | - | deterministic core shared by relay, keep, and other services |

### Step-Driven Raft Consensus Core

Services can embed one deterministic Raft core while keeping transport,
storage, and state-machine concerns outside the consensus library.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `raft_core`.
- Gate — behavior: `cargo test -p raft-core` - consensus and snapshot behavior
  coverage
- Gate: `cargo test -p raft-core`
- Source: `libs/raft-core/src/lib.rs`
- Evidence: `cargo test -p raft-core`; libs/raft-core/src/lib.rs
