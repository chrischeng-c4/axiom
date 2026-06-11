// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-src-config-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
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

use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct ClusterConfig {
    pub shard_count: u32,
    pub replicas_per_shard: u32,
    pub voter_count: u32,
    pub pod_name: String,
}

impl ClusterConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            shard_count: parse_env("SHARD_COUNT")?,
            replicas_per_shard: parse_env("REPLICAS_PER_SHARD")?,
            voter_count: parse_env("VOTER_COUNT")?,
            pod_name: env::var("POD_NAME").context("POD_NAME not set")?,
        })
    }

    pub fn pod_ordinal(&self) -> Result<u32> {
        let (_, suffix) = self
            .pod_name
            .rsplit_once('-')
            .context("POD_NAME has no '-<ordinal>' suffix")?;
        suffix
            .parse()
            .with_context(|| format!("POD_NAME ordinal '{suffix}' is not a u32"))
    }

    pub fn shard_index(&self) -> Result<u32> {
        Ok(self.pod_ordinal()? % self.shard_count)
    }

    pub fn replica_index(&self) -> Result<u32> {
        Ok(self.pod_ordinal()? / self.shard_count)
    }

    pub fn is_voter(&self) -> Result<bool> {
        Ok(self.replica_index()? < self.voter_count)
    }
}

fn parse_env(key: &str) -> Result<u32> {
    env::var(key)
        .with_context(|| format!("{key} not set"))?
        .parse()
        .with_context(|| format!("{key} must be a u32"))
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

// </HANDWRITE>
