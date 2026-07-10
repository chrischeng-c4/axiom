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
}
// CODEGEN-END
