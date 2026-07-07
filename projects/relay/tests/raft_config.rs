// SPEC-MANAGED: projects/relay/tech-design/logic/adopt-raft-host-relaystatemachine-auto-mode-ha-drop-hand-rolled.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:d0cba133" tracker="pending-tracker" reason="Topology smoke (#544): relay derives node id / membership / peer URLs from the standard downward-API quartet via raft_host::cluster (the hand-derived local ordinal math is deleted); replica_mode is off without cluster env; RELAY_PEERS overrides peer DNS for a local multi-node group."
//! Topology smoke (#544): relay's cluster shape comes from
//! `raft_host::cluster` and the STANDARD downward-API env — the hand-derived
//! ordinal/peer-DNS module is gone, per CONTRIBUTING's "never re-derive the
//! ordinal math locally".

use std::sync::Mutex;

use raft_host::cluster::{replica_mode, ClusterTopology};

// The standard env vars are process-global; serialize the env tests.
static ENV_LOCK: Mutex<()> = Mutex::new(());

const QUARTET: [(&str, &str); 4] = [
    ("POD_NAME", "relay-1"),
    ("SHARD_COUNT", "1"),
    ("REPLICAS_PER_SHARD", "3"),
    ("VOTER_COUNT", "3"),
];

fn clear_env() {
    for (k, _) in QUARTET {
        std::env::remove_var(k);
    }
    std::env::remove_var("RELAY_PEERS");
}

/// Auto-mode switch: no cluster env (or a single replica) = single-node;
/// REPLICAS_PER_SHARD > 1 = replica/HA.
#[test]
fn replica_mode_is_off_without_cluster_env() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_env();
    assert!(!replica_mode(), "no env = single-node");
    std::env::set_var("REPLICAS_PER_SHARD", "1");
    assert!(!replica_mode(), "one replica = single-node");
    std::env::set_var("REPLICAS_PER_SHARD", "3");
    assert!(replica_mode(), "scaled out = replica/HA");
    clear_env();
}

/// The serve path's exact derivation: pod relay-1 in a 1-shard/3-replica
/// group is node 1 with voters {0,1,2}; peer URLs use the headless-Service
/// DNS template, and RELAY_PEERS replaces them for a local multi-node group.
#[test]
fn topology_derives_from_standard_env_via_raft_host() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_env();
    for (k, v) in QUARTET {
        std::env::set_var(k, v);
    }

    let t = ClusterTopology::from_env("relay", "relay", 8080, "RELAY_PEERS").unwrap();
    assert_eq!(t.node_id, 1, "node id = replica index from the pod ordinal");
    assert_eq!(t.membership.voters, vec![0, 1, 2]);
    assert!(t.membership.learners.is_empty());
    assert_eq!(t.peers.len(), 2, "self excluded");
    assert_eq!(t.peers[&0], "http://relay-0.relay:8080");
    assert_eq!(t.peers[&2], "http://relay-2.relay:8080");

    // RELAY_PEERS local override replaces the DNS-derived addresses.
    std::env::set_var(
        "RELAY_PEERS",
        "127.0.0.1:7101,127.0.0.1:7102,127.0.0.1:7103",
    );
    let t = ClusterTopology::from_env("relay", "relay", 8080, "RELAY_PEERS").unwrap();
    assert_eq!(t.peers[&0], "http://127.0.0.1:7101");
    assert_eq!(t.peers[&2], "http://127.0.0.1:7103");
    clear_env();
}
// HANDWRITE-END
