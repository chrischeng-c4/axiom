use std::sync::Mutex;

use raft_runtime::{ClusterTopology, MembershipPolicy, ReplicaHostBuilder};

static ENV: Mutex<()> = Mutex::new(());

struct ExactlyThree;

impl MembershipPolicy for ExactlyThree {
    fn validate(&self, topology: &ClusterTopology) -> anyhow::Result<()> {
        anyhow::ensure!(
            topology.replicas_per_shard == 3
                && topology.membership.voters.len() == 3
                && topology.membership.learners.is_empty(),
            "service requires exactly three voters"
        );
        Ok(())
    }
}

#[test]
fn service_policy_validates_shared_topology_without_owning_startup() {
    let _guard = ENV.lock().unwrap();
    for (key, value) in [
        ("SHARD_COUNT", "1"),
        ("REPLICAS_PER_SHARD", "3"),
        ("VOTER_COUNT", "3"),
        ("POD_NAME", "sift-store-0"),
    ] {
        std::env::set_var(key, value);
    }
    let builder = ReplicaHostBuilder::new(
        "sift",
        "sift-store-headless",
        7381,
        "SIFT_PEERS",
        "https",
        ExactlyThree,
    )
    .unwrap();
    let topology = builder.topology().unwrap();
    assert_eq!(topology.node_id, 0);
    assert_eq!(topology.membership.voters, vec![0, 1, 2]);

    std::env::set_var("VOTER_COUNT", "2");
    let error = builder.topology().unwrap_err().to_string();
    assert!(error.contains("exactly three voters"), "{error}");
    for key in [
        "SHARD_COUNT",
        "REPLICAS_PER_SHARD",
        "VOTER_COUNT",
        "POD_NAME",
    ] {
        std::env::remove_var(key);
    }
}
