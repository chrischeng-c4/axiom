// HANDWRITE-BEGIN gap="missing-generator:unit-test:direct-install-statefulset" tracker="#1809" reason="Offline contract for Tape's direct Kustomize baseline: durable singleton, restricted pod, and explicit operator-only raft-HA boundary."
//! Static contracts for Tape's direct Kubernetes install path.

use serde_yaml::Value;

fn yaml(input: &str) -> Value {
    serde_yaml::from_str(input).expect("manifest parses")
}

#[test]
fn direct_base_is_a_durable_singleton_not_a_fake_raft_scaleout() {
    let statefulset = yaml(include_str!("../k8s/base/statefulset.yaml"));
    assert_eq!(statefulset["kind"], "StatefulSet");
    assert_eq!(statefulset["spec"]["replicas"], 1);
    assert_eq!(statefulset["spec"]["serviceName"], "tape-headless");
    assert_eq!(
        statefulset["spec"]["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"],
        "10Gi"
    );

    let container = &statefulset["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(container["command"][0], "tape");
    assert_eq!(container["command"][1], "serve");
    assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);
    assert_eq!(
        container["securityContext"]["allowPrivilegeEscalation"],
        false
    );

    let env = container["env"].as_sequence().expect("container env");
    for (name, value) in [
        ("SHARD_COUNT", "1"),
        ("REPLICAS_PER_SHARD", "1"),
        ("VOTER_COUNT", "1"),
        ("TAPE_PEER_SERVICE", "tape-headless"),
        ("TAPE_DATA_DIR", "/data"),
    ] {
        assert!(
            env.iter()
                .any(|entry| { entry["name"] == name && entry["value"] == value }),
            "missing {name}={value}"
        );
    }
}

#[test]
fn prod_profile_uses_projected_auth_without_a_direct_hpa() {
    let prod = include_str!("../k8s/overlays/prod/kustomization.yaml");
    assert!(prod.contains("TAPE_TOKEN_REGISTRY_FILE"));
    assert!(prod.contains("token-registry.json"));
    assert!(prod.contains("../../components/observability"));
    assert!(prod.contains("../../components/network-policy"));
    assert!(!prod.contains("HorizontalPodAutoscaler"));
    assert!(!prod.contains("kind: HPA"));
}
// HANDWRITE-END
