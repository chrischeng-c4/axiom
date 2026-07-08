# raft-core

## Brief

`raft-core` is a self-contained, step-driven Raft consensus core. It is
transport- and storage-agnostic so services can supply their network,
persistence, and state machine layers.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Step-Driven Raft Consensus Core | - | implemented | verified | smoke | ready | deterministic core shared by relay, keep, and other services |

### Step-Driven Raft Consensus Core

ID: step-driven-raft-consensus-core
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `raft_core`.
EC Dimensions: behavior: `cargo test -p raft-core` - consensus and snapshot behavior coverage
Required Verification: smoke
Promise:
Services can embed one deterministic Raft core while keeping transport,
storage, and state-machine concerns outside the consensus library.
Gate Inventory: `cargo test -p raft-core`; libs/raft-core/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| step-driven-raft-consensus-core-contract | epic | - | implemented | verified | smoke | `cargo test -p raft-core`; libs/raft-core/src/lib.rs |
