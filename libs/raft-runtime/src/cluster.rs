// CODEGEN-BEGIN
//! k8s-native cluster topology + auto-mode for the raft host.
//!
//! Every raft_core service derives the same thing from the StatefulSet downward
//! API: which mode to run (single-node vs replica/HA), this node's id, the
//! group membership, and the peer URLs. This module centralizes it so services
//! compose it instead of hand-rolling the ordinal math + peer-DNS each time.

use std::collections::HashMap;

use anyhow::{bail, Context, Result};

use crate::{Membership, NodeId};

/// Whether the StatefulSet runs in replica/HA mode: `true` when
/// `REPLICAS_PER_SHARD > 1`. A single replica — or no cluster context (the env
/// unset, e.g. local dev) — is single-node. This is the **auto-mode** switch: a
/// service defaults to single-node and turns on raft only when k8s scales it out.
pub fn replica_mode() -> bool {
    std::env::var("REPLICAS_PER_SHARD")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1)
        > 1
}

/// Guard a controller that is still backed by raft-runtime's startup-static
/// membership. Changing the StatefulSet replica count without a replicated
/// membership transition is unsafe: existing members and new pods would run
/// with different quorum sets. Callers must keep the replica layer unchanged
/// until raft-core/raft-runtime expose the joint-consensus workflow.
pub fn ensure_static_membership_unchanged(current: u32, desired: u32) -> Result<()> {
    if current != desired {
        bail!(
            "unsafe replica transition {current}->{desired}: raft-runtime membership is static; complete a replicated membership transition before changing StatefulSet replicas"
        );
    }
    Ok(())
}

/// The scalar shard/replica/voter derivation from the standard downward-API
/// quartet (`SHARD_COUNT`, `REPLICAS_PER_SHARD`, `VOTER_COUNT`, `POD_NAME`) —
/// the piece [`ClusterTopology::from_env`] shares with a caller that only
/// needs the scalars, not peer URLs (e.g. lumen's `ClusterConfig`, which
/// stays compiled outside the `raft-wal` feature; #1002).
#[derive(Debug, Clone)]
pub struct ClusterDims {
    pub shard_count: u32,
    pub replicas_per_shard: u32,
    pub voter_count: u32,
    pub pod_name: String,
}

impl ClusterDims {
    /// Read the standard downward-API quartet.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            shard_count: parse_env("SHARD_COUNT")?,
            replicas_per_shard: parse_env("REPLICAS_PER_SHARD")?,
            voter_count: parse_env("VOTER_COUNT")?,
            pod_name: std::env::var("POD_NAME").context("POD_NAME not set")?,
        })
    }

    /// The trailing `-<N>` ordinal in `pod_name` — the StatefulSet identity.
    pub fn pod_ordinal(&self) -> Result<u32> {
        let (_, suffix) = self
            .pod_name
            .rsplit_once('-')
            .context("POD_NAME has no '-<ordinal>' suffix")?;
        suffix
            .parse()
            .with_context(|| format!("POD_NAME ordinal '{suffix}' is not a u32"))
    }

    /// The part of `pod_name` before the ordinal — i.e. the StatefulSet's own
    /// name, which is also the peer-DNS prefix (`<sts>-<n>.<headless>`).
    ///
    /// This is the same `rsplit_once` that [`Self::pod_ordinal`] uses, keeping
    /// the other half instead of discarding it. That matters: a pod trusts the
    /// ordinal from this string to decide *who it is*, so the prefix from the
    /// identical parse is exactly as trustworthy for deciding *who to call*.
    /// A caller cannot know this value — an operator names the StatefulSet
    /// after the custom resource, so only the pod's own downward-API `POD_NAME`
    /// carries it.
    pub fn pod_prefix(&self) -> Result<&str> {
        let (prefix, _) = self
            .pod_name
            .rsplit_once('-')
            .context("POD_NAME has no '-<ordinal>' suffix")?;
        if prefix.is_empty() {
            bail!("POD_NAME '{}' has an empty StatefulSet prefix", self.pod_name);
        }
        Ok(prefix)
    }

    /// `ordinal % shard_count` — which shard this pod belongs to.
    pub fn shard_index(&self) -> Result<u32> {
        Ok(self.pod_ordinal()? % self.shard_count)
    }

    /// `ordinal / shard_count` — this pod's replica index within its shard
    /// (== the raft node id).
    pub fn replica_index(&self) -> Result<u32> {
        Ok(self.pod_ordinal()? / self.shard_count)
    }

    /// Whether this replica votes (`replica_index < voter_count`); the rest
    /// are learners.
    pub fn is_voter(&self) -> Result<bool> {
        Ok(self.replica_index()? < self.voter_count)
    }
}

/// The StatefulSet pod ordinal for `replica` within a shard
/// (`replica * shard_count + shard_index`) — the peer-DNS math shared by
/// every peer enumeration: [`ClusterTopology::from_env`]'s peer URLs and a
/// caller's own richer per-peer record (e.g. lumen's `RaftGroup`/`PeerAddr`).
pub fn peer_ordinal(shard_count: u32, shard_index: u32, replica: u32) -> u32 {
    replica * shard_count + shard_index
}

/// Parse a `LUMEN_PEERS`-style override env var (`host[:port],host[:port],...`,
/// empty entries filtered) into an `index -> host[:port]` override list.
/// Empty when `env_var` is unset — callers then use the DNS-derived
/// addresses unmodified. Shared by [`ClusterTopology::from_env`] and any
/// caller enumerating its own peer records with the same override contract.
pub fn parse_peer_overrides(env_var: &str) -> Vec<String> {
    std::env::var(env_var)
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// One raft group's topology, derived from the StatefulSet downward API.
#[derive(Debug, Clone)]
pub struct ClusterTopology {
    /// This node's id within its shard's group = the replica index.
    pub node_id: NodeId,
    /// Voters `0..voter_count`, learners the rest.
    pub membership: Membership,
    /// Peer base URLs (`NodeId → http://host:port`), excluding self.
    pub peers: HashMap<NodeId, String>,
    pub replicas_per_shard: u32,
    pub shard_index: u32,
}

impl ClusterTopology {
    /// Build from the standard downward-API env (`POD_NAME`, `SHARD_COUNT`,
    /// `REPLICAS_PER_SHARD`, `VOTER_COUNT`) and a peer-DNS template
    /// (`<prefix>-<ordinal>.<headless_service>:<peer_port>`). `peers_override` is
    /// the name of an env var (e.g. `LUMEN_PEERS`) holding `host[:port],...` that
    /// replaces the DNS addresses — for running a multi-node group on one machine.
    ///
    /// `fallback_prefix` is only consulted when `POD_NAME` carries no usable
    /// StatefulSet name; see [`Self::from_env_with_scheme`] for why the pod's own
    /// name wins.
    pub fn from_env(
        fallback_prefix: &str,
        headless_service: &str,
        peer_port: u16,
        peers_override: &str,
    ) -> Result<Self> {
        Self::from_env_with_scheme(
            fallback_prefix,
            headless_service,
            peer_port,
            peers_override,
            "http",
        )
    }

    /// TLS-aware variant used by stateful services serving Raft on a dedicated
    /// mTLS peer port. Only `http` and `https` are accepted so a malformed
    /// scheme cannot silently weaken or redirect peer traffic.
    ///
    /// The peer-DNS prefix comes from [`ClusterDims::pod_prefix`] — the pod's
    /// own StatefulSet name — and `fallback_prefix` is used only if `POD_NAME`
    /// carries none. Callers used to pass their binary name here as the real
    /// prefix, which is right only when the custom resource happens to be named
    /// after the binary: an operator names the StatefulSet after the CR, so a CR
    /// named `quorum` produces pods `quorum-0`/`quorum-1` while the binary was
    /// addressing `lumen-1.<headless>`. That name does not resolve, no
    /// `RequestVote` is ever delivered, and every voter campaigns forever
    /// without ever hearing a peer — a silent, permanent loss of quorum whose
    /// only symptom is that no leader appears. Deriving the prefix from
    /// `POD_NAME` makes it correct for any CR name, and identical to the old
    /// behavior when the two already agreed.
    pub fn from_env_with_scheme(
        fallback_prefix: &str,
        headless_service: &str,
        peer_port: u16,
        peers_override: &str,
        scheme: &str,
    ) -> Result<Self> {
        if !matches!(scheme, "http" | "https") {
            bail!("raft peer URL scheme must be http or https");
        }
        let dims = ClusterDims::from_env()?;
        let shard_count = dims.shard_count;
        let replicas_per_shard = dims.replicas_per_shard;
        let voter_count = dims.voter_count;
        if shard_count == 0 {
            bail!("SHARD_COUNT must be greater than zero");
        }
        if replicas_per_shard == 0 {
            bail!("REPLICAS_PER_SHARD must be greater than zero");
        }
        if voter_count == 0 || voter_count > replicas_per_shard {
            bail!("VOTER_COUNT must be in 1..=REPLICAS_PER_SHARD");
        }
        let shard_index = dims.shard_index()?;
        let node_id = dims.replica_index()? as NodeId;
        if node_id >= replicas_per_shard as NodeId {
            bail!("POD_NAME ordinal resolves outside REPLICAS_PER_SHARD");
        }

        // The peer-DNS prefix is this pod's own StatefulSet name. Only fall back
        // to the caller's guess if `POD_NAME` somehow carries none — and say so,
        // because at that point peer addressing rests on an assumption rather
        // than on observed identity.
        let prefix = match dims.pod_prefix() {
            Ok(p) => p,
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    fallback = fallback_prefix,
                    "raft peer prefix not derivable from POD_NAME; falling back to the caller's"
                );
                fallback_prefix
            }
        };
        if prefix != fallback_prefix {
            // Not a problem — it is the normal case for any CR not named after
            // the binary — but it is the one fact that makes peer URLs resolve,
            // so it belongs in the startup log next to node_id and peers.
            tracing::debug!(
                derived = prefix,
                caller_default = fallback_prefix,
                "raft peer prefix taken from POD_NAME's StatefulSet name"
            );
        }

        // pod ordinal → (shard, replica) is pure integer math, so peers are found
        // via headless DNS with no discovery service. `index N → replica N`.
        let overrides = parse_peer_overrides(peers_override);

        let mut peers = HashMap::new();
        for replica in 0..replicas_per_shard {
            let id = replica as NodeId;
            if id == node_id {
                continue;
            }
            let url = match overrides.get(replica as usize) {
                Some(addr) if addr.contains(':') => format!("{scheme}://{addr}"),
                Some(addr) => format!("{scheme}://{addr}:{peer_port}"),
                None => {
                    let ordinal = peer_ordinal(shard_count, shard_index, replica);
                    format!("{scheme}://{prefix}-{ordinal}.{headless_service}:{peer_port}")
                }
            };
            peers.insert(id, url);
        }

        let membership = Membership {
            voters: (0..voter_count as NodeId).collect(),
            learners: (voter_count as NodeId..replicas_per_shard as NodeId).collect(),
        };
        Ok(Self {
            node_id,
            membership,
            peers,
            replicas_per_shard,
            shard_index,
        })
    }
}

fn parse_env(key: &str) -> Result<u32> {
    std::env::var(key)
        .with_context(|| format!("{key} not set"))?
        .parse()
        .with_context(|| format!("{key} must be a u32"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // The standard env vars are process-global; serialize the env tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn replica_mode_defaults_to_single_node() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("REPLICAS_PER_SHARD");
        assert!(!replica_mode());
        std::env::set_var("REPLICAS_PER_SHARD", "1");
        assert!(!replica_mode());
        std::env::set_var("REPLICAS_PER_SHARD", "3");
        assert!(replica_mode());
        std::env::remove_var("REPLICAS_PER_SHARD");
    }

    #[test]
    fn static_membership_rejects_replica_delta() {
        ensure_static_membership_unchanged(3, 3).unwrap();
        let err = ensure_static_membership_unchanged(1, 3).unwrap_err();
        assert!(err.to_string().contains("replicated membership transition"));
    }

    #[test]
    fn topology_from_env_with_local_override() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("SHARD_COUNT", "1");
        std::env::set_var("REPLICAS_PER_SHARD", "3");
        std::env::set_var("VOTER_COUNT", "3");
        std::env::set_var("POD_NAME", "svc-1");
        std::env::set_var("SVC_PEERS", "10.0.0.0:9001,10.0.0.1:9002,10.0.0.2:9003");
        let t = ClusterTopology::from_env("svc", "svc-headless", 7000, "SVC_PEERS").unwrap();
        assert_eq!(t.node_id, 1);
        assert_eq!(t.membership.voters, vec![0, 1, 2]);
        // self (id 1) excluded; peers point at the override addresses.
        assert_eq!(t.peers.get(&0).unwrap(), "http://10.0.0.0:9001");
        assert_eq!(t.peers.get(&2).unwrap(), "http://10.0.0.2:9003");
        assert!(!t.peers.contains_key(&1));
        let tls = ClusterTopology::from_env_with_scheme(
            "svc",
            "svc-headless",
            7000,
            "SVC_PEERS",
            "https",
        )
        .unwrap();
        assert_eq!(tls.peers.get(&0).unwrap(), "https://10.0.0.0:9001");
        assert!(ClusterTopology::from_env_with_scheme(
            "svc",
            "svc-headless",
            7000,
            "SVC_PEERS",
            "ftp",
        )
        .is_err());
        for k in [
            "SHARD_COUNT",
            "REPLICAS_PER_SHARD",
            "VOTER_COUNT",
            "POD_NAME",
            "SVC_PEERS",
        ] {
            std::env::remove_var(k);
        }
    }

    #[test]
    fn peer_dns_prefix_follows_the_pod_not_the_callers_binary_name() {
        // The regression this exists for: an operator names the StatefulSet
        // after the custom resource, so a CR named `quorum` produces pods
        // `quorum-0`/`quorum-1` while the binary passed its own name, `lumen`.
        // The resulting peer URL `lumen-1.<headless>` is NXDOMAIN, so no
        // RequestVote is ever delivered and every voter campaigns forever.
        // Asserted on the URL rather than on a bool, because the failure was
        // never an error — it was a well-formed address for a host that does
        // not exist.
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("SHARD_COUNT", "1");
        std::env::set_var("REPLICAS_PER_SHARD", "2");
        std::env::set_var("VOTER_COUNT", "2");
        std::env::set_var("POD_NAME", "quorum-0");
        std::env::remove_var("LUMEN_PEERS");

        let t =
            ClusterTopology::from_env("lumen", "quorum-headless", 7373, "LUMEN_PEERS").unwrap();
        assert_eq!(t.node_id, 0);
        assert_eq!(
            t.peers.get(&1).unwrap(),
            "http://quorum-1.quorum-headless:7373",
            "peer URL must name the pod's own StatefulSet, not the caller's binary"
        );

        // The pre-existing deployments this must not disturb: when the CR *is*
        // named after the binary, the derived prefix equals the passed one and
        // the URL is byte-identical to what it always was.
        std::env::set_var("POD_NAME", "lumen-0");
        let same =
            ClusterTopology::from_env("lumen", "lumen-headless", 7373, "LUMEN_PEERS").unwrap();
        assert_eq!(
            same.peers.get(&1).unwrap(),
            "http://lumen-1.lumen-headless:7373"
        );

        for k in ["SHARD_COUNT", "REPLICAS_PER_SHARD", "VOTER_COUNT", "POD_NAME"] {
            std::env::remove_var(k);
        }
    }

    fn dims(shard_count: u32, replicas_per_shard: u32, voter_count: u32, pod: &str) -> ClusterDims {
        ClusterDims {
            shard_count,
            replicas_per_shard,
            voter_count,
            pod_name: pod.into(),
        }
    }

    #[test]
    fn pod_prefix_is_the_statefulset_name_and_rejects_a_nameless_pod() {
        // Hyphenated CR names are ordinary (`my-search-cluster-3`), so the split
        // has to be the LAST hyphen, matching `pod_ordinal`'s.
        assert_eq!(dims(1, 1, 1, "quorum-0").pod_prefix().unwrap(), "quorum");
        assert_eq!(
            dims(1, 1, 1, "my-search-cluster-3").pod_prefix().unwrap(),
            "my-search-cluster"
        );
        // No prefix to derive: refuse rather than emit `http://-1.headless`.
        assert!(dims(1, 1, 1, "-0").pod_prefix().is_err());
        assert!(dims(1, 1, 1, "nohyphen").pod_prefix().is_err());
    }

    #[test]
    fn cluster_dims_derives_shard_and_replica_from_pod_ordinal() {
        // 3 shards × 3 replicas: pod-7 → shard 1, replica 2.
        let d = dims(3, 3, 3, "svc-7");
        assert_eq!(d.pod_ordinal().unwrap(), 7);
        assert_eq!(d.shard_index().unwrap(), 1);
        assert_eq!(d.replica_index().unwrap(), 2);
        assert!(d.is_voter().unwrap());

        let d = dims(3, 3, 2, "svc-8");
        assert_eq!(d.shard_index().unwrap(), 2);
        assert_eq!(d.replica_index().unwrap(), 2);
        assert!(
            !d.is_voter().unwrap(),
            "replica 2 is a learner when voter_count=2"
        );
    }

    #[test]
    fn cluster_dims_pod_ordinal_rejects_bad_suffix() {
        assert!(dims(3, 3, 3, "svc-").pod_ordinal().is_err());
        assert!(dims(3, 3, 3, "svc-abc").pod_ordinal().is_err());
        assert!(dims(3, 3, 3, "svc").pod_ordinal().is_err());
    }

    #[test]
    fn peer_ordinal_matches_replica_times_shard_count_plus_shard() {
        assert_eq!(peer_ordinal(3, 1, 2), 7); // 2*3+1
        assert_eq!(peer_ordinal(1, 0, 4), 4);
    }

    #[test]
    fn parse_peer_overrides_splits_trims_and_filters_empty() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("TEST_PEERS");
        assert!(parse_peer_overrides("TEST_PEERS").is_empty());

        std::env::set_var("TEST_PEERS", " a:1, b:2 ,,c:3");
        assert_eq!(
            parse_peer_overrides("TEST_PEERS"),
            vec!["a:1".to_string(), "b:2".to_string(), "c:3".to_string()]
        );
        std::env::remove_var("TEST_PEERS");
    }
}
// CODEGEN-END
