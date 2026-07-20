#![cfg(feature = "operator")]
// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-operator" tracker="#766" reason="Pure CRD/operator render conformance for stateful HA, security, and scheduled backups."
use defer::operator::{self, Defer};

fn production() -> Defer {
    serde_json::from_value(serde_json::json!({
        "apiVersion": "defer.dev/v1alpha1",
        "kind": "Defer",
        "metadata": {"name": "jobs", "namespace": "payments", "uid": "u-1"},
        "spec": {
            "image": "registry/defer:1",
            "imagePullPolicy": "Always",
            "replicasPerShard": 3,
            "voterCount": 3,
            "storage": "100Gi",
            "auth": "required",
            "tokensSecret": "defer-tokens",
            "targetSigningSecret": "target-signing",
            "targetSigningKeyId": "active",
            "peerTlsSecret": "peer-tls",
            "backup": {
                "schedule": "0 */6 * * *",
                "destination": "s3://backups/defer",
                "retentionSecs": 604800,
                "adminTokenSecret": "backup-token"
            },
            "resources": {"cpu": "2", "memory": "8Gi"}
        }
    }))
    .unwrap()
}

#[test]
fn production_render_composes_shared_stateful_primitives() {
    let objects = operator::render::render(&production());
    let kind = |wanted: &str| {
        objects
            .iter()
            .find(|object| object["kind"] == wanted)
            .unwrap()
    };
    let stateful = kind("StatefulSet");
    assert_eq!(stateful["metadata"]["namespace"], "payments");
    assert_eq!(stateful["spec"]["replicas"], 3);
    assert_eq!(
        stateful["spec"]["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"],
        "100Gi"
    );
    let encoded = serde_json::to_string(stateful).unwrap();
    for required in [
        "DEFER_DATA_DIR",
        "DEFER_TOKEN_REGISTRY_FILE",
        "DEFER_TARGET_SIGNING_SECRET_FILE",
        "DEFER_PEER_MTLS",
        "REPLICAS_PER_SHARD",
        "/healthz",
        "/readyz",
    ] {
        assert!(encoded.contains(required), "missing {required}");
    }
    assert!(objects.iter().any(|object| object["kind"] == "Service"));
    assert!(objects
        .iter()
        .any(|object| object["kind"] == "PodDisruptionBudget"));
    let cron = kind("CronJob");
    assert_eq!(cron["spec"]["schedule"], "0 */6 * * *");
    assert!(serde_json::to_string(cron)
        .unwrap()
        .contains("s3://backups/defer"));
}

#[test]
fn generated_crd_is_structural_and_includes_backup_and_signing() {
    let yaml = operator::crd_yaml();
    assert!(yaml.contains("name: defers.defer.dev"));
    assert!(yaml.contains("targetSigningSecret"));
    assert!(yaml.contains("backup"));
    assert!(!yaml.contains("format: uint64"));
    assert!(!yaml.contains("format: uint32"));
}
// HANDWRITE-END
