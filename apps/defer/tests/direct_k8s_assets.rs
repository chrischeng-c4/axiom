// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-direct-k8s" tracker="#766" reason="Offline contract for Defer direct Kustomize singleton and operator-owned HA boundary."
use serde_yaml::Value;

fn yaml(input: &str) -> Value {
    serde_yaml::from_str(input).expect("manifest parses")
}

#[test]
fn direct_base_is_a_restricted_durable_singleton() {
    let statefulset = yaml(include_str!("../k8s/base/statefulset.yaml"));
    assert_eq!(statefulset["kind"], "StatefulSet");
    assert_eq!(statefulset["spec"]["replicas"], 1);
    assert_eq!(statefulset["spec"]["serviceName"], "defer-headless");
    let container = &statefulset["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(container["command"][0], "defer");
    assert_eq!(container["command"][1], "serve");
    assert_eq!(container["ports"][0]["containerPort"], 7141);
    assert_eq!(container["ports"][1]["containerPort"], 7142);
    assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);
    let env = container["env"].as_sequence().unwrap();
    assert!(env.iter().any(|entry| entry["name"] == "DEFER_LOG_FORMAT"));
    let config = yaml(include_str!("../k8s/base/configmap.yaml"));
    assert_eq!(config["data"]["DEFER_LOG_FORMAT"], "json");
    for (name, value) in [
        ("SHARD_COUNT", "1"),
        ("REPLICAS_PER_SHARD", "1"),
        ("VOTER_COUNT", "1"),
        ("DEFER_PEER_SERVICE", "defer-headless"),
        ("DEFER_RAFT_PORT", "7142"),
        ("DEFER_DATA_DIR", "/data"),
    ] {
        assert!(env
            .iter()
            .any(|entry| entry["name"] == name && entry["value"] == value));
    }
}

#[test]
fn prod_profile_uses_security_components_without_voter_hpa() {
    let prod = include_str!("../k8s/overlays/prod/kustomization.yaml");
    assert!(prod.contains("DEFER_TOKEN_REGISTRY_FILE"));
    assert!(prod.contains("../../components/observability"));
    assert!(prod.contains("../../components/network-policy"));
    assert!(!prod.contains("HorizontalPodAutoscaler"));
    assert!(!prod.contains("kind: HPA"));
}
// HANDWRITE-END
