---
id: libs-raft-host-src-state-machine-rs
summary: Lossless rust-source-unit coverage for `libs/raft-host/src/state_machine.rs`.
capability_refs:
  - id: shared-raft-host-driver
    role: primary
    claim: shared-raft-host-driver-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Raft Host library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/raft-host/src/state_machine.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/raft-host/src/state_machine.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Command` | libs/raft-host/src/state_machine.rs | type | pub | 7 | pub type Command = Vec<u8>; |
| `RaftStateMachine` | libs/raft-host/src/state_machine.rs | trait | pub | 17 | pub trait RaftStateMachine: Send + Sync + 'static { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The `RaftStateMachine` a consumer supplies to [`crate::RaftHost`].

use raft_core::Index;

/// Opaque committed-entry bytes (raft_core's `RaftEntry.command`). The host never
/// looks inside — the state machine encodes/decodes its own commands.
pub type Command = Vec<u8>;

/// The consumer's replicated state machine. The host owns the **only** applier:
/// every committed entry is fed to [`apply`](RaftStateMachine::apply) exactly
/// once, in index order, on every node, from a single task under the node lock.
/// [`snapshot`](RaftStateMachine::snapshot) / [`restore`](RaftStateMachine::restore)
/// bound the log (compaction) and let a lagging/fresh replica catch up.
///
/// Implementors are `&self` interior-mutable (engines are `Arc<_>` with internal
/// locks); the host holds an `Arc<dyn RaftStateMachine>`.
pub trait RaftStateMachine: Send + Sync + 'static {
    /// Apply one committed command at `index` (1-based, strictly increasing, once
    /// per entry). `index` equals the raft log index (for lumen, the WAL seq).
    /// An `Err` is logged by the host and the entry is treated as applied
    /// (no-op) so the log keeps advancing — the implementor must still advance
    /// its own [`applied_index`](RaftStateMachine::applied_index) past `index`.
    fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()>;

    /// Serialize the full state as of the last applied index. The host ships
    /// these bytes via `InstallSnapshot` and stores them through `node.compact`.
    fn snapshot(&self) -> anyhow::Result<Vec<u8>>;

    /// Replace the entire state from snapshot bytes (a follower installing a
    /// leader's snapshot, or cold-start). After this, [`applied_index`] must
    /// return the snapshot's index.
    fn restore(&self, snapshot: &[u8]) -> anyhow::Result<()>;

    /// Highest index durably applied by this state machine (survives restart).
    /// Drives the host's commit-wait (read-your-write) and the idempotency floor.
    fn applied_index(&self) -> Index;
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/raft-host/src/state_machine.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/raft-host/src/state_machine.rs` captured during libs codegen standardization.
```
