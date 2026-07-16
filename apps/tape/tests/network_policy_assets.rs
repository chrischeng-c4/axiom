// HANDWRITE-BEGIN gap="missing-generator:unit-test:6f3fecd5" tracker="#1593" reason="Parse the static component and assert its server selector, namespace-label peers, and TCP-only ingress boundary. generator gap: missing-generator:kubernetes-network-policy-test."
//! Offline contracts for Tape's opt-in Kubernetes ingress boundary.

use serde_yaml::Value;

fn yaml(input: &str) -> Value {
    serde_yaml::from_str(input).expect("network policy manifest parses")
}

#[test]
fn component_is_explicitly_opt_in() {
    let component = yaml(include_str!(
        "../k8s/components/network-policy/kustomization.yaml"
    ));
    assert_eq!(component["apiVersion"], "kustomize.config.k8s.io/v1alpha1");
    assert_eq!(component["kind"], "Component");
    let resources = component["resources"]
        .as_sequence()
        .expect("component resources are a sequence");
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0], "networkpolicy.yaml");
}

#[test]
fn policy_selects_only_tape_servers_and_has_no_unrestricted_peer() {
    let policy = yaml(include_str!(
        "../k8s/components/network-policy/networkpolicy.yaml"
    ));
    assert_eq!(policy["apiVersion"], "networking.k8s.io/v1");
    assert_eq!(policy["kind"], "NetworkPolicy");
    assert_eq!(policy["spec"]["podSelector"]["matchLabels"]["app"], "tape");
    assert_eq!(
        policy["spec"]["podSelector"]["matchLabels"]["role"],
        "server"
    );
    let policy_types = policy["spec"]["policyTypes"]
        .as_sequence()
        .expect("policy types are a sequence");
    assert_eq!(policy_types.len(), 1);
    assert_eq!(policy_types[0], "Ingress");

    let ingress = policy["spec"]["ingress"]
        .as_sequence()
        .expect("ingress rules are a sequence");
    assert_eq!(
        ingress.len(),
        3,
        "only sibling Tape peers, labeled clients, and Prometheus are allowed"
    );

    let raft = &ingress[0];
    let raft_ports = raft["ports"].as_sequence().expect("raft rule has ports");
    assert_eq!(raft_ports.len(), 2);
    assert_eq!(raft_ports[0]["protocol"], "TCP");
    assert_eq!(raft_ports[0]["port"], 7137);
    assert_eq!(raft_ports[1]["protocol"], "TCP");
    assert_eq!(raft_ports[1]["port"], 7138);
    let raft_peers = raft["from"].as_sequence().expect("raft rule has peers");
    assert_eq!(raft_peers.len(), 1);
    assert_eq!(raft_peers[0]["podSelector"]["matchLabels"]["app"], "tape");
    assert_eq!(
        raft_peers[0]["podSelector"]["matchLabels"]["role"],
        "server"
    );
    assert!(
        raft_peers[0]["namespaceSelector"].is_null(),
        "the sibling-pod selector is intentionally limited to Tape's own namespace"
    );

    for rule in &ingress[1..] {
        let ports = rule["ports"].as_sequence().expect("rule has ports");
        assert_eq!(ports.len(), 1, "each peer has exactly one allowed port");
        assert_eq!(ports[0]["protocol"], "TCP");
        assert_eq!(ports[0]["port"], 7137);

        let peers = rule["from"].as_sequence().expect("rule has peers");
        assert_eq!(peers.len(), 1, "no broad fallback peer is allowed");
        assert!(
            peers[0]["namespaceSelector"]["matchLabels"].is_mapping(),
            "every allowed peer must name an opt-in namespace label"
        );
    }

    assert_eq!(
        ingress[1]["from"][0]["namespaceSelector"]["matchLabels"]["tape.cclab.dev/client-access"],
        "true"
    );
    assert!(
        ingress[1]["from"][0]["podSelector"].is_null(),
        "client access is bounded by the explicit namespace label"
    );
    assert_eq!(
        ingress[2]["from"][0]["namespaceSelector"]["matchLabels"]
            ["tape.cclab.dev/monitoring-access"],
        "true"
    );
    assert_eq!(
        ingress[2]["from"][0]["podSelector"]["matchLabels"]["app.kubernetes.io/name"],
        "prometheus"
    );
}
// HANDWRITE-END
