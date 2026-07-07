---
id: projects-lumen-src-config-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/config.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/config.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ClusterConfig` | projects/lumen/src/config.rs | struct | pub | 25 |  |
| `from_env` | projects/lumen/src/config.rs | function | pub | 56 | from_env() -> Result<Self> |
| `is_voter` | projects/lumen/src/config.rs | function | pub | 72 | is_voter(&self) -> Result<bool> |
| `pod_ordinal` | projects/lumen/src/config.rs | function | pub | 60 | pod_ordinal(&self) -> Result<u32> |
| `replica_index` | projects/lumen/src/config.rs | function | pub | 68 | replica_index(&self) -> Result<u32> |
| `shard_index` | projects/lumen/src/config.rs | function | pub | 64 | shard_index(&self) -> Result<u32> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-config-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Runtime config — sourced from env so it can be wired through the K8s
//! ConfigMap without any rebuild.
//!
//! Two orthogonal dimensions:
//!   shard_count          — how data is partitioned (collection_id hash)
//!   replicas_per_shard   — Raft group size per shard
//!   voter_count          — first N replicas vote; the rest are learners
//!
//! pod ordinal → (shard_index, replica_index) is pure integer math, so
//! peers can be found via headless DNS with no extra discovery service.
//!
//! The env keys and the ordinal/shard/replica/voter derivation are the same
//! StatefulSet downward-API math every raft_core service needs, and live in
//! `libs/raft-host::cluster::ClusterDims` (#1002); this module is a thin
//! adapter that keeps lumen's own `ClusterConfig` type (compiled unconditionally,
//! unlike the raft-wal-only peer/DNS wiring in `raft.rs`) while delegating the
//! actual math so it can't drift from `raft_host::cluster::ClusterTopology`.

use anyhow::Result;

#[derive(Debug, Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-config-rs.md#source
pub struct ClusterConfig {
    pub shard_count: u32,
    pub replicas_per_shard: u32,
    pub voter_count: u32,
    pub pod_name: String,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-config-rs.md#source
impl From<raft_host::cluster::ClusterDims> for ClusterConfig {
    fn from(d: raft_host::cluster::ClusterDims) -> Self {
        Self {
            shard_count: d.shard_count,
            replicas_per_shard: d.replicas_per_shard,
            voter_count: d.voter_count,
            pod_name: d.pod_name,
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-config-rs.md#source
impl From<ClusterConfig> for raft_host::cluster::ClusterDims {
    fn from(c: ClusterConfig) -> Self {
        Self {
            shard_count: c.shard_count,
            replicas_per_shard: c.replicas_per_shard,
            voter_count: c.voter_count,
            pod_name: c.pod_name,
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-config-rs.md#source
impl ClusterConfig {
    pub fn from_env() -> Result<Self> {
        Ok(raft_host::cluster::ClusterDims::from_env()?.into())
    }

    pub fn pod_ordinal(&self) -> Result<u32> {
        raft_host::cluster::ClusterDims::from(self.clone()).pod_ordinal()
    }

    pub fn shard_index(&self) -> Result<u32> {
        raft_host::cluster::ClusterDims::from(self.clone()).shard_index()
    }

    pub fn replica_index(&self) -> Result<u32> {
        raft_host::cluster::ClusterDims::from(self.clone()).replica_index()
    }

    pub fn is_voter(&self) -> Result<bool> {
        raft_host::cluster::ClusterDims::from(self.clone()).is_voter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(
        shard_count: u32,
        replicas_per_shard: u32,
        voter_count: u32,
        pod: &str,
    ) -> ClusterConfig {
        ClusterConfig {
            shard_count,
            replicas_per_shard,
            voter_count,
            pod_name: pod.into(),
        }
    }

    #[test]
    fn pod_ordinal_extracts_trailing_int() {
        assert_eq!(cfg(3, 3, 3, "lumen-0").pod_ordinal().unwrap(), 0);
        assert_eq!(cfg(3, 3, 3, "lumen-7").pod_ordinal().unwrap(), 7);
        assert_eq!(cfg(3, 3, 3, "lumen-42").pod_ordinal().unwrap(), 42);
    }

    #[test]
    fn pod_ordinal_rejects_bad_suffix() {
        assert!(cfg(3, 3, 3, "lumen-").pod_ordinal().is_err());
        assert!(cfg(3, 3, 3, "lumen-abc").pod_ordinal().is_err());
        assert!(cfg(3, 3, 3, "lumen-3-foo").pod_ordinal().is_err());
    }

    #[test]
    fn pod_ordinal_rejects_no_dash() {
        assert!(cfg(3, 3, 3, "lumen").pod_ordinal().is_err());
    }

    #[test]
    fn shard_and_replica_indices_partition_correctly() {
        // 3 shards × 3 replicas: pod-0/3/6 → shard 0, pod-1/4/7 → shard 1, etc.
        let c = cfg(3, 3, 3, "lumen-7");
        assert_eq!(c.shard_index().unwrap(), 1);
        assert_eq!(c.replica_index().unwrap(), 2);
        assert!(c.is_voter().unwrap());

        let c = cfg(3, 3, 2, "lumen-8");
        assert_eq!(c.shard_index().unwrap(), 2);
        assert_eq!(c.replica_index().unwrap(), 2);
        assert!(
            !c.is_voter().unwrap(),
            "replica 2 is a learner when voter_count=2"
        );
    }

    #[test]
    fn from_env_round_trips() {
        // env is process-global; share a mutex with the tls tests via
        // a local one here.
        use std::sync::Mutex;
        static LOCK: Mutex<()> = Mutex::new(());
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());

        unsafe {
            std::env::set_var("SHARD_COUNT", "3");
            std::env::set_var("REPLICAS_PER_SHARD", "3");
            std::env::set_var("VOTER_COUNT", "3");
            std::env::set_var("POD_NAME", "lumen-4");
        }
        let cfg = ClusterConfig::from_env().unwrap();
        assert_eq!(cfg.shard_count, 3);
        assert_eq!(cfg.replicas_per_shard, 3);
        assert_eq!(cfg.voter_count, 3);
        assert_eq!(cfg.pod_name, "lumen-4");
        unsafe {
            std::env::remove_var("SHARD_COUNT");
            std::env::remove_var("REPLICAS_PER_SHARD");
            std::env::remove_var("VOTER_COUNT");
            std::env::remove_var("POD_NAME");
        }
    }

    #[test]
    fn from_env_errors_on_missing_var() {
        use std::sync::Mutex;
        static LOCK: Mutex<()> = Mutex::new(());
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("SHARD_COUNT");
            std::env::remove_var("REPLICAS_PER_SHARD");
            std::env::remove_var("VOTER_COUNT");
            std::env::remove_var("POD_NAME");
        }
        assert!(ClusterConfig::from_env().is_err());
    }

    #[test]
    fn from_env_errors_on_non_u32() {
        use std::sync::Mutex;
        static LOCK: Mutex<()> = Mutex::new(());
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("SHARD_COUNT", "not-a-number");
            std::env::set_var("REPLICAS_PER_SHARD", "3");
            std::env::set_var("VOTER_COUNT", "3");
            std::env::set_var("POD_NAME", "lumen-0");
        }
        assert!(ClusterConfig::from_env().is_err());
        unsafe {
            std::env::remove_var("SHARD_COUNT");
            std::env::remove_var("REPLICAS_PER_SHARD");
            std::env::remove_var("VOTER_COUNT");
            std::env::remove_var("POD_NAME");
        }
    }
}
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/config.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/config.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
