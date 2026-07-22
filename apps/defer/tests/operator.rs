#![cfg(feature = "operator")]
// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-operator" tracker="#766" reason="Pure CRD/operator render conformance for stateful HA, security, and scheduled backups."
use defer::operator::{self, Defer};
use serde_json::{json, Value};

fn object<'a>(objects: &'a [Value], kind: &str, name: &str) -> &'a Value {
    objects
        .iter()
        .find(|object| object["kind"] == kind && object["metadata"]["name"] == name)
        .unwrap_or_else(|| panic!("missing {kind}/{name}"))
}

fn named<'a>(array: &'a Value, name: &str) -> &'a Value {
    array
        .as_array()
        .unwrap_or_else(|| panic!("expected array while finding {name}"))
        .iter()
        .find(|value| value["name"] == name)
        .unwrap_or_else(|| panic!("missing named entry {name}"))
}

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

// <HANDWRITE gap="missing-generator:unit-test" tracker="#2220" reason="Own the exact six-object production graph, three-replica StatefulSet relationships, connected security secrets, and backup CronJob oracle.">
#[test]
fn production_render_composes_shared_stateful_primitives() {
    let objects = operator::render::render(&production());
    assert_eq!(objects.len(), 6, "exact operator-owned resource graph");
    let selector = json!({
        "app.kubernetes.io/name": "defer",
        "app.kubernetes.io/instance": "jobs",
        "app.kubernetes.io/component": "server"
    });
    let stateful = object(&objects, "StatefulSet", "jobs");
    assert_eq!(stateful["metadata"]["namespace"], "payments");
    assert_eq!(stateful["spec"]["replicas"], 3);
    assert_eq!(stateful["spec"]["serviceName"], "jobs-headless");
    assert_eq!(stateful["spec"]["selector"]["matchLabels"], selector);
    assert_eq!(
        stateful["spec"]["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"],
        "100Gi"
    );
    assert_eq!(
        stateful["spec"]["volumeClaimTemplates"][0]["metadata"]["name"],
        "data"
    );
    assert_eq!(
        stateful["spec"]["volumeClaimTemplates"][0]["spec"]["accessModes"],
        json!(["ReadWriteOnce"])
    );
    let container = &stateful["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(
        container["ports"],
        json!([
            {"name": "http", "containerPort": 7141, "protocol": "TCP"},
            {"name": "raft", "containerPort": 7142, "protocol": "TCP"}
        ])
    );
    assert_eq!(
        container["readinessProbe"],
        json!({"httpGet": {"path": "/readyz", "port": "http"}, "periodSeconds": 5})
    );
    assert_eq!(
        container["livenessProbe"],
        json!({"httpGet": {"path": "/healthz", "port": "http"}, "periodSeconds": 15})
    );
    assert_eq!(
        container["startupProbe"],
        json!({"httpGet": {"path": "/healthz", "port": "http"}, "periodSeconds": 5, "failureThreshold": 120})
    );
    assert_eq!(
        named(&container["volumeMounts"], "data")["mountPath"],
        "/data"
    );
    assert_eq!(named(&container["volumeMounts"], "data")["readOnly"], false);

    let headless = object(&objects, "Service", "jobs-headless");
    assert_eq!(headless["spec"]["clusterIP"], "None");
    assert_eq!(headless["spec"]["publishNotReadyAddresses"], true);
    assert_eq!(headless["spec"]["selector"], selector);
    assert_eq!(
        headless["spec"]["ports"],
        json!([
            {"name": "http", "port": 7141, "targetPort": "http", "protocol": "TCP"},
            {"name": "raft", "port": 7142, "targetPort": "raft", "protocol": "TCP"}
        ])
    );
    let client = object(&objects, "Service", "jobs");
    assert_eq!(client["spec"]["type"], "ClusterIP");
    assert_eq!(client["spec"]["selector"], selector);
    assert_eq!(
        client["spec"]["ports"],
        json!([{"name": "http", "port": 7141, "targetPort": "http", "protocol": "TCP"}])
    );
    let pdb = object(&objects, "PodDisruptionBudget", "jobs");
    assert_eq!(pdb["spec"]["maxUnavailable"], 1);
    assert_eq!(pdb["spec"]["selector"]["matchLabels"], selector);

    let volumes = &stateful["spec"]["template"]["spec"]["volumes"];
    assert_eq!(
        named(volumes, "defer-token-registry"),
        &json!({
            "name": "defer-token-registry",
            "secret": {"secretName": "defer-tokens", "items": [{"key": "token-registry.json", "path": "token-registry.json"}]}
        })
    );
    assert_eq!(
        named(&container["volumeMounts"], "defer-token-registry"),
        &json!({"name": "defer-token-registry", "mountPath": "/var/run/secrets/defer", "readOnly": true})
    );
    assert_eq!(
        named(volumes, "defer-target-signing"),
        &json!({
            "name": "defer-target-signing",
            "secret": {"secretName": "target-signing", "items": [{"key": "key", "path": "key"}]}
        })
    );
    assert_eq!(
        named(&container["volumeMounts"], "defer-target-signing"),
        &json!({"name": "defer-target-signing", "mountPath": "/var/run/secrets/defer-target", "readOnly": true})
    );
    assert_eq!(
        named(volumes, "defer-peer-tls"),
        &json!({
            "name": "defer-peer-tls",
            "secret": {"secretName": "peer-tls", "items": [
                {"key": "tls.crt", "path": "tls.crt"},
                {"key": "tls.key", "path": "tls.key"},
                {"key": "ca.crt", "path": "ca.crt"}
            ]}
        })
    );
    assert_eq!(
        named(&container["volumeMounts"], "defer-peer-tls"),
        &json!({"name": "defer-peer-tls", "mountPath": "/var/run/secrets/defer-peer", "readOnly": true})
    );
    for (name, value) in [
        ("DEFER_AUTH", "required"),
        (
            "DEFER_TOKEN_REGISTRY_FILE",
            "/var/run/secrets/defer/token-registry.json",
        ),
        ("DEFER_TARGET_SIGNING_KEY_ID", "active"),
        (
            "DEFER_TARGET_SIGNING_SECRET_FILE",
            "/var/run/secrets/defer-target/key",
        ),
        ("DEFER_PEER_MTLS", "on"),
        ("DEFER_PEER_TLS_CERT", "/var/run/secrets/defer-peer/tls.crt"),
        ("DEFER_PEER_TLS_KEY", "/var/run/secrets/defer-peer/tls.key"),
        ("DEFER_PEER_TLS_CA", "/var/run/secrets/defer-peer/ca.crt"),
    ] {
        assert_eq!(named(&container["env"], name)["value"], value, "{name}");
    }

    let cron = object(&objects, "CronJob", "jobs-backup");
    assert_eq!(cron["spec"]["schedule"], "0 */6 * * *");
    let backup = &cron["spec"]["jobTemplate"]["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(
        backup["args"],
        json!([
            "backup",
            "--url",
            "http://jobs.payments.svc.cluster.local:7141",
            "--dest",
            "s3://backups/defer",
            "--retention-secs",
            "604800"
        ])
    );
    assert_eq!(
        named(&backup["env"], "DEFER_TOKEN")["valueFrom"]["secretKeyRef"],
        json!({"name": "backup-token", "key": "token"})
    );
}
// </HANDWRITE>

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
