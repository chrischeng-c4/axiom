// SPEC-MANAGED: projects/relay/tech-design/logic/k8s-manifests-ordinal-role-kind-failover-smoke-raft-ha-layer-2.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:fb773cca" tracker="pending-tracker" reason="Pure cluster-config derivation for k8s: ordinal_from_hostname('relay-N') -> Option<NodeId>; peer_urls(service, namespace, port, n, self_id) -> HashMap<NodeId,String> of headless-Service DNS URLs excluding self; RaftClusterConfig::from_env() assembling node_id/membership/peers/subject/data_dir/port from env vars. No external dependency."
//! Derive a node's Raft cluster config from its Kubernetes environment.
//!
//! On a StatefulSet the pod hostname is `relay-<ordinal>` and the replica count
//! is injected as an env var; from those two facts a node knows its id, its
//! voter/learner role ([`crate::raft::auto_membership`]), and every peer's
//! stable DNS name behind the headless Service. These helpers are pure so the
//! k8s wiring is covered by fast unit tests.

use std::collections::HashMap;

use crate::config::FsyncPolicy;
use crate::raft::{auto_membership, Membership, NodeId};

/// Parse the StatefulSet ordinal from a pod hostname like `relay-3` → `3`.
/// Returns `None` if the suffix after the last `-` is not a number.
///
/// @spec projects/relay/tech-design/logic/k8s-manifests-ordinal-role-kind-failover-smoke-raft-ha-layer-2.md#logic
pub fn ordinal_from_hostname(hostname: &str) -> Option<NodeId> {
    hostname
        .rsplit_once('-')
        .and_then(|(_, n)| n.parse::<NodeId>().ok())
}

/// Build the peer URL map (every node except `self_id`) from the headless
/// Service DNS: `http://relay-<i>.<service>.<namespace>.svc.cluster.local:<port>`.
///
/// @spec projects/relay/tech-design/logic/k8s-manifests-ordinal-role-kind-failover-smoke-raft-ha-layer-2.md#logic
pub fn peer_urls(
    service: &str,
    namespace: &str,
    port: u16,
    n: u64,
    self_id: NodeId,
) -> HashMap<NodeId, String> {
    (0..n)
        .filter(|i| *i != self_id)
        .map(|i| {
            (
                i,
                format!("http://relay-{i}.{service}.{namespace}.svc.cluster.local:{port}"),
            )
        })
        .collect()
}

/// Everything a `relay-raft` node needs to join its group.
pub struct RaftClusterConfig {
    pub node_id: NodeId,
    pub membership: Membership,
    pub peers: HashMap<NodeId, String>,
    pub subject: String,
    pub data_dir: String,
    pub port: u16,
    pub fsync: FsyncPolicy,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

impl RaftClusterConfig {
    /// Assemble the config from the (downward-API) environment.
    ///
    /// @spec projects/relay/tech-design/logic/k8s-manifests-ordinal-role-kind-failover-smoke-raft-ha-layer-2.md#logic
    pub fn from_env() -> RaftClusterConfig {
        let hostname = env_or("HOSTNAME", "relay-0");
        let node_id = ordinal_from_hostname(&hostname).unwrap_or(0);
        let n: u64 = env_or("RELAY_REPLICAS", "1").parse().unwrap_or(1);
        let service = env_or("RELAY_SERVICE", "relay");
        let namespace = env_or("RELAY_NAMESPACE", "default");
        let port: u16 = env_or("RELAY_PORT", "8080").parse().unwrap_or(8080);
        RaftClusterConfig {
            node_id,
            membership: auto_membership(n),
            peers: peer_urls(&service, &namespace, port, n, node_id),
            subject: env_or("RELAY_SUBJECT", "events"),
            data_dir: env_or("RELAY_DATA_DIR", "/data"),
            port,
            fsync: FsyncPolicy::Always,
        }
    }
}
// HANDWRITE-END
