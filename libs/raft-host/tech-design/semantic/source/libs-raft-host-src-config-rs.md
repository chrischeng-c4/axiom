---
id: libs-raft-host-src-config-rs
summary: Lossless rust-source-unit coverage for `libs/raft-host/src/config.rs`.
capability_refs:
  - id: shared-raft-host-driver
    role: primary
    claim: shared-raft-host-driver-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Raft Host library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/raft-host/src/config.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/raft-host/src/config.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SnapshotPolicy` | libs/raft-host/src/config.rs | enum | pub | 7 | pub enum SnapshotPolicy { |
| `HostConfig` | libs/raft-host/src/config.rs | struct | pub | 20 | pub struct HostConfig { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! [`RaftHost`](crate::RaftHost) tuning.

use std::time::Duration;

/// When the host captures a state-machine snapshot and compacts the raft log.
#[derive(Clone, Copy, Debug)]
pub enum SnapshotPolicy {
    /// Never compact (the log grows; fine for log-broker state machines with no
    /// meaningful snapshot, e.g. relay).
    Disabled,
    /// Compact when `applied_index - snapshot_index >= n`.
    EveryEntries(u64),
    /// The host never auto-compacts; the consumer drives it (e.g. lumen's
    /// periodic RDB snapshotter calls `snapshot_and_compact`).
    External,
}

/// Host timing + snapshot policy.
#[derive(Clone, Copy, Debug)]
pub struct HostConfig {
    /// Logical tick (election/heartbeat clock).
    pub tick: Duration,
    /// Fast outbox pump (ships replies-driven work under the election timeout).
    pub pump: Duration,
    /// Peer RPC timeout.
    pub rpc_timeout: Duration,
    /// How long `propose` waits for its entry to apply before erroring.
    pub propose_timeout: Duration,
    /// Auto-compaction policy.
    pub snapshot: SnapshotPolicy,
}

impl Default for HostConfig {
    fn default() -> Self {
        HostConfig {
            tick: Duration::from_millis(20),
            pump: Duration::from_millis(5),
            rpc_timeout: Duration::from_millis(400),
            propose_timeout: Duration::from_secs(10),
            snapshot: SnapshotPolicy::Disabled,
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/raft-host/src/config.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/raft-host/src/config.rs` captured during libs codegen standardization.
```
