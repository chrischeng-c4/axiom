// SPEC-MANAGED: projects/relay/tech-design/logic/k8s-manifests-ordinal-role-kind-failover-smoke-raft-ha-layer-2.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:d0cba133" tracker="pending-tracker" reason="Tests: ordinal_from_hostname parses relay-<n> and rejects others; peer_urls builds the headless-Service DNS URLs for all peers except self; membership marks the trailing even ordinal a learner."
//! k8s cluster-config derivation (#139): a node figures out its id, peers and
//! voter/learner role from its hostname + replica count alone.

use relay::{auto_membership, ordinal_from_hostname, peer_urls};

#[test]
fn ordinal_parsed_from_statefulset_hostname() {
    assert_eq!(ordinal_from_hostname("relay-0"), Some(0));
    assert_eq!(ordinal_from_hostname("relay-3"), Some(3));
    // namespaced / FQDN-style prefixes still resolve by the trailing ordinal.
    assert_eq!(ordinal_from_hostname("my-relay-12"), Some(12));
    // non-numeric suffixes are rejected.
    assert_eq!(ordinal_from_hostname("relay-x"), None);
    assert_eq!(ordinal_from_hostname("standalone"), None);
}

#[test]
fn peer_urls_use_headless_dns_and_exclude_self() {
    let peers = peer_urls("relay", "default", 8080, 3, 1);
    assert_eq!(peers.len(), 2, "all peers except self");
    assert!(!peers.contains_key(&1), "self excluded");
    assert_eq!(
        peers[&0],
        "http://relay-0.relay.default.svc.cluster.local:8080"
    );
    assert_eq!(
        peers[&2],
        "http://relay-2.relay.default.svc.cluster.local:8080"
    );
}

#[test]
fn membership_makes_the_trailing_even_node_a_learner() {
    // N=4 -> voters {0,1,2}, learner {3} (voter count stays odd).
    let m = auto_membership(4);
    assert_eq!(m.voters, vec![0, 1, 2]);
    assert_eq!(m.learners, vec![3]);
    // N=3 -> all voters, no learner.
    assert_eq!(auto_membership(3).learners.len(), 0);
}
// HANDWRITE-END
