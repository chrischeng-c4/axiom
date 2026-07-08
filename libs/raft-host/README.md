# raft-host

## Brief

`raft-host` drives `raft-core` for caller-supplied state machines over h2c peer
transport, with snapshot, compaction, and read-your-write propose support.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Raft Host Driver | - | implemented | verified | smoke | ready | h2c peer host for raft-core state machines |

### Shared Raft Host Driver

ID: shared-raft-host-driver
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `raft_host`.
EC Dimensions: behavior: `cargo test -p raft-host` - host, config, store, and read consistency coverage
Required Verification: smoke
Promise:
Services can host Raft state machines through a shared h2c driver instead of
duplicating peer transport and read consistency plumbing.
Gate Inventory: `cargo test -p raft-host`; libs/raft-host/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-raft-host-driver-contract | epic | - | implemented | verified | smoke | `cargo test -p raft-host`; libs/raft-host/src/lib.rs |
