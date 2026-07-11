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
| `ClusterConfig` | projects/lumen/src/config.rs | struct | pub | 27 |  |
| `from_env` | projects/lumen/src/config.rs | function | pub | 60 | from_env() -> Result<Self> |
| `is_voter` | projects/lumen/src/config.rs | function | pub | 76 | is_voter(&self) -> Result<bool> |
| `pod_ordinal` | projects/lumen/src/config.rs | function | pub | 64 | pod_ordinal(&self) -> Result<u32> |
| `replica_index` | projects/lumen/src/config.rs | function | pub | 72 | replica_index(&self) -> Result<u32> |
| `shard_index` | projects/lumen/src/config.rs | function | pub | 68 | shard_index(&self) -> Result<u32> |
| `shard_map_from_env` | projects/lumen/src/config.rs | function | pub | 95 | shard_map_from_env(shard_count: u32) -> Result<VirtualBucketShardMap> |
| `fan_in_shard_count` | projects/lumen/src/config.rs | function | pub | 144 | fan_in_shard_count(explicit: Option<u32>, loaded_dirs: usize) -> u32 |
| `check_fan_in_shard_count` | projects/lumen/src/config.rs | function | pub | 156 | check_fan_in_shard_count(map: &VirtualBucketShardMap, loaded_dirs: usize) -> Result<()> |
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

use anyhow::{Context, Result};

use crate::routing::{VirtualBucketShardMap, DEFAULT_VIRTUAL_BUCKET_COUNT};

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

/// Live shard-routing map from the same ConfigMap env the operator renders
/// (`serving_configmap` in `operator/render.rs`): `SHARD_MAP_VERSION` /
/// `SHARD_MAP_ASSIGNMENTS` / `VIRTUAL_BUCKET_COUNT`. Mirrors
/// `ShardMapSpec`'s "empty assignments means balanced" contract exactly —
/// `SHARD_MAP_ASSIGNMENTS` unset (the ConfigMap key is only written once the
/// operator has committed a real split; see `serving_configmap`) falls back
/// to today's `bucket % shard_count` balanced default, so a pod started
/// before any reshard, or with no shard-map env at all (plain `lumen serve`
/// outside k8s), routes exactly as it always has (#1384 AC4). `shard_count`
/// is the caller's already-resolved physical shard count (`ClusterConfig::
/// shard_count` / `ServeArgs::shard_count`), not re-read from env here, so
/// this stays usable without the full raft `ClusterConfig` (e.g. the
/// non-raft `--search-shard-segment-dirs` read-shard-fan-in path).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-config-rs.md#source
pub fn shard_map_from_env(shard_count: u32) -> Result<VirtualBucketShardMap> {
    let version = match std::env::var("SHARD_MAP_VERSION") {
        Ok(raw) => raw
            .trim()
            .parse::<u64>()
            .with_context(|| format!("SHARD_MAP_VERSION={raw:?} is not a valid u64"))?,
        Err(_) => 0,
    };
    let virtual_bucket_count = match std::env::var("VIRTUAL_BUCKET_COUNT") {
        Ok(raw) => raw
            .trim()
            .parse::<u32>()
            .with_context(|| format!("VIRTUAL_BUCKET_COUNT={raw:?} is not a valid u32"))?,
        Err(_) => DEFAULT_VIRTUAL_BUCKET_COUNT,
    };
    match std::env::var("SHARD_MAP_ASSIGNMENTS") {
        Ok(raw) if !raw.trim().is_empty() => {
            let assignments = raw
                .split(',')
                .map(|s| {
                    s.trim().parse::<u32>().with_context(|| {
                        format!("SHARD_MAP_ASSIGNMENTS entry {s:?} is not a valid u32")
                    })
                })
                .collect::<Result<Vec<u32>>>()?;
            VirtualBucketShardMap::new(version, assignments, shard_count.max(1))
        }
        _ => VirtualBucketShardMap::balanced(version, virtual_bucket_count, shard_count.max(1)),
    }
}

/// Physical shard count the segment-dirs fan-in path
/// (`lumen serve --search-shard-segment-dirs a,b,c`) should route across.
///
/// `explicit` is `ServeArgs::shard_count`, a clap `Option<u32>` with
/// `env = "SHARD_COUNT"` and **no** `default_value_t` — that shape is
/// deliberate: it's the only way to tell "the operator/user actually set
/// `--shard-count`/`SHARD_COUNT`" (`Some`) from "nobody set anything"
/// (`None`) at the type level, since a `u32` field with a `default_value_t`
/// can't distinguish an explicit `--shard-count 1`/`SHARD_COUNT=1` from
/// clap's own default. When `explicit` is `Some`, it is honored as-is
/// (still fed through `shard_map_from_env` below, so `SHARD_MAP_*` env
/// overrides still apply on top). When it is `None`, the count defaults to
/// `loaded_dirs`, restoring `EngineShardSearch::new`'s original
/// derive-from-loaded-dirs behavior (#1398 R4: the prior call site fed
/// clap's old `default_value_t = 1` straight into `shard_map_from_env`,
/// so `a,b,c` with `SHARD_COUNT` unset silently built a 1-shard map and
/// searched only dir `a` on routed queries).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-config-rs.md#source
pub fn fan_in_shard_count(explicit: Option<u32>, loaded_dirs: usize) -> u32 {
    explicit.unwrap_or(loaded_dirs as u32)
}

/// Startup guard for the segment-dirs fan-in path: the shard map's declared
/// `physical_shard_count` must equal the number of loaded
/// `--search-shard-segment-dirs` roots, or routed queries would silently
/// reach only a subset of shards (#1398 R4 — e.g. an explicit
/// `SHARD_COUNT` that doesn't match the actual dir count). Names both
/// numbers in the error instead of continuing to serve with an
/// inconsistent map.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-config-rs.md#source
pub fn check_fan_in_shard_count(map: &VirtualBucketShardMap, loaded_dirs: usize) -> Result<()> {
    let declared = map.physical_shard_count() as usize;
    if declared != loaded_dirs {
        anyhow::bail!(
            "shard map physical_shard_count ({declared}) does not match the number of \
             loaded --search-shard-segment-dirs ({loaded_dirs}); set SHARD_COUNT/--shard-count \
             to {loaded_dirs} or fix the loaded dirs"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Every test below that mutates process env (`ClusterConfig::from_env`'s
    // SHARD_COUNT/REPLICAS_PER_SHARD/VOTER_COUNT/POD_NAME and
    // `shard_map_from_env`'s SHARD_MAP_VERSION/SHARD_MAP_ASSIGNMENTS/
    // VIRTUAL_BUCKET_COUNT) shares this one lock — env vars are
    // process-global, and `cargo test`'s default parallel runner would
    // otherwise interleave two tests' `set_var`/`remove_var` calls (a
    // per-function-local `static LOCK` does *not* serialize across
    // functions; each fn body owns a distinct static).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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

    // ---- shard_map_from_env (#1384) ------------------------------------

    fn clear_shard_map_env() {
        unsafe {
            std::env::remove_var("SHARD_MAP_VERSION");
            std::env::remove_var("SHARD_MAP_ASSIGNMENTS");
            std::env::remove_var("VIRTUAL_BUCKET_COUNT");
        }
    }

    #[test]
    fn shard_map_from_env_falls_back_to_balanced_when_unset() {
        // #1384 AC4: no shard-map env at all (today's default deployment,
        // and every deployment before the operator ever commits a real
        // split) must produce byte-identical routing to the pre-#1384
        // `VirtualBucketShardMap::balanced` construction.
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_shard_map_env();

        let map = shard_map_from_env(4).unwrap();
        let expected = VirtualBucketShardMap::balanced(0, DEFAULT_VIRTUAL_BUCKET_COUNT, 4).unwrap();
        assert_eq!(map, expected);
        assert_eq!(map.version(), 0);
        assert_eq!(map.virtual_bucket_count(), DEFAULT_VIRTUAL_BUCKET_COUNT);
        assert_eq!(map.physical_shard_count(), 4);
    }

    #[test]
    fn shard_map_from_env_falls_back_to_balanced_when_assignments_blank() {
        // The ConfigMap key is written unconditionally for
        // SHARD_MAP_VERSION/VIRTUAL_BUCKET_COUNT but SHARD_MAP_ASSIGNMENTS
        // is only ever present once assignments are non-empty
        // (`serving_configmap`) — an empty/whitespace value must not be
        // parsed as "one bucket assigned to shard \"\"".
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_shard_map_env();
        unsafe {
            std::env::set_var("SHARD_MAP_VERSION", "2");
            std::env::set_var("VIRTUAL_BUCKET_COUNT", "16");
            std::env::set_var("SHARD_MAP_ASSIGNMENTS", "  ");
        }

        let map = shard_map_from_env(4).unwrap();
        assert_eq!(map, VirtualBucketShardMap::balanced(2, 16, 4).unwrap());
        clear_shard_map_env();
    }

    #[test]
    fn shard_map_from_env_honors_explicit_assignments() {
        // #1384 AC1: an explicit SHARD_MAP_ASSIGNMENTS (the cutover-flipped
        // map a driver split commits) is what a pod started after it must
        // route by — not the balanced default.
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_shard_map_env();
        unsafe {
            std::env::set_var("SHARD_MAP_VERSION", "1");
            std::env::set_var("VIRTUAL_BUCKET_COUNT", "8");
            std::env::set_var("SHARD_MAP_ASSIGNMENTS", "1,1,1,1,0,0,0,0");
        }

        let map = shard_map_from_env(2).unwrap();
        assert_eq!(map.version(), 1);
        assert_eq!(map.virtual_bucket_count(), 8);
        assert_eq!(map.physical_shard_count(), 2);
        assert_eq!(map.assignment_for_bucket(0), Some(1));
        assert_eq!(map.assignment_for_bucket(4), Some(0));
        assert_ne!(
            map,
            VirtualBucketShardMap::balanced(1, 8, 2).unwrap(),
            "explicit assignments must override the balanced default"
        );
        clear_shard_map_env();
    }

    #[test]
    fn shard_map_from_env_rejects_out_of_range_assignment() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_shard_map_env();
        unsafe {
            std::env::set_var("SHARD_MAP_ASSIGNMENTS", "0,1,2");
        }

        assert!(
            shard_map_from_env(2).is_err(),
            "bucket assigned to shard 2 with only 2 physical shards must error"
        );
        clear_shard_map_env();
    }

    #[test]
    fn shard_map_from_env_rejects_non_numeric_version() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_shard_map_env();
        unsafe {
            std::env::set_var("SHARD_MAP_VERSION", "not-a-number");
        }

        assert!(shard_map_from_env(2).is_err());
        clear_shard_map_env();
    }

    // ---- fan_in_shard_count / check_fan_in_shard_count (#1398 R4) ------

    #[test]
    fn fan_in_shard_count_derives_from_loaded_dirs_when_unset() {
        // AC3: `--search-shard-segment-dirs a,b,c` with no SHARD_COUNT must
        // route across all three dirs, not clap's old default of 1.
        assert_eq!(fan_in_shard_count(None, 3), 3);
        assert_eq!(fan_in_shard_count(None, 1), 1);
    }

    #[test]
    fn fan_in_shard_count_honors_explicit_value() {
        // An explicit --shard-count/SHARD_COUNT (including an explicit `1`,
        // which is indistinguishable from clap's removed default only by
        // being `Some`) always wins over the loaded-dir count.
        assert_eq!(fan_in_shard_count(Some(1), 3), 1);
        assert_eq!(fan_in_shard_count(Some(5), 3), 5);
    }

    #[test]
    fn check_fan_in_shard_count_passes_when_counts_match() {
        let map = VirtualBucketShardMap::balanced(0, 16, 3).unwrap();
        assert!(check_fan_in_shard_count(&map, 3).is_ok());
    }

    #[test]
    fn check_fan_in_shard_count_fails_fast_on_mismatch() {
        // AC3: a mismatched explicit count (here: a map built for 3 shards
        // but only 2 dirs actually loaded) must fail startup with a clear
        // message naming both numbers, not silently under-route.
        let map = VirtualBucketShardMap::balanced(0, 16, 3).unwrap();
        let err = check_fan_in_shard_count(&map, 2).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains('3'),
            "error should name the declared count: {msg}"
        );
        assert!(
            msg.contains('2'),
            "error should name the loaded-dir count: {msg}"
        );
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
