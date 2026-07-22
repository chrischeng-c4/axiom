// HANDWRITE-BEGIN gap="sift-deployment-cli-tests" tracker="1606" reason="Verify all Dockerfile and layered Kubernetes artifact commands render expected contracts."
use std::{
    collections::HashMap,
    process::{Command, Output},
};

use axiom_operator::{ManagedService, ReadyFacts};
use serde::Deserialize;
use serde_json::{json, Value};
use sift::operator::Sift;

fn sift_output(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sift"))
        .args(args)
        .output()
        .expect("run sift command")
}

fn sift(args: &[&str]) -> String {
    let output = sift_output(args);
    assert!(output.status.success(), "{args:?}: {output:?}");
    String::from_utf8(output.stdout).expect("utf-8 deployment output")
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="1675" reason="Verify collector rendering preserves the least-privilege node-log contract.">
#[test]
fn layered_deployment_cli_renders_all_artifact_planes() {
    let dockerfile = sift(&["dockerfile", "render", "--variant", "source"]);
    assert!(dockerfile.contains("FROM rust:"));
    assert!(dockerfile.contains("COPY --chown=65532:65532"));
    assert!(dockerfile.contains("next:"));

    let release_dockerfile = sift(&[
        "dockerfile",
        "render",
        "--variant",
        "release",
        "--version",
        "sift@0.1.0",
    ]);
    assert!(release_dockerfile.contains("SIFT_VERSION=0.1.0"));
    assert!(release_dockerfile.contains("x86_64-unknown-linux-gnu"));
    assert!(release_dockerfile.contains("aarch64-unknown-linux-gnu"));
    assert!(release_dockerfile.contains("install -m 755"));

    let crd = sift(&["k8s", "crd", "render"]);
    assert!(crd.contains("kind: CustomResourceDefinition"));
    assert!(crd.contains("sifts.sift.axiom.dev"));

    let operator = sift(&[
        "k8s",
        "operator",
        "render",
        "--namespace",
        "sift-system",
        "--image",
        "example.invalid/sift-operator:gke-test",
    ]);
    assert!(operator.contains("kind: Namespace"));
    assert!(operator.contains("kind: Deployment"));
    assert!(operator.contains("sift k8s operator run"));
    assert!(operator.contains("runAsNonRoot: true"));
    assert!(operator.contains("example.invalid/sift-operator:gke-test"));
    assert!(operator.contains("resources: [\"leases\"]"));
    assert!(operator.contains("resources: [\"cronjobs\"]"));
    assert!(operator.contains("serviceaccounts"));

    let instance = sift(&["k8s", "instance", "render", "--profile", "dev"]);
    assert!(instance.contains("kind: Sift"));
    assert!(instance.contains("replicasPerShard: 1"));
    assert!(instance.contains("auth: \"off\""));
    assert!(instance.contains("sift:0.1.0"));

    let collector = sift(&[
        "k8s",
        "collector",
        "render",
        "--namespace",
        "observability",
        "--image",
        "example.invalid/sift:1.2.3",
    ]);
    assert!(collector.contains("kind: DaemonSet"));
    assert!(collector.contains("namespace: observability"));
    assert!(collector.contains("image: example.invalid/sift:1.2.3"));
    assert!(collector.contains("automountServiceAccountToken: false"));
    assert!(collector.contains("path: /var/log/pods"));
    assert!(collector.contains("mountPath: /var/log/pods\n              readOnly: true"));
    assert!(collector.contains("path: /var/lib/sift-collector"));
    assert!(collector.contains("secretKeyRef:"));
    assert!(collector.contains("configMapKeyRef:"));
    assert!(collector.contains("fieldPath: spec.nodeName"));
    assert!(collector.contains("chown 0:0 /var/lib/sift-collector"));
    assert!(collector.contains("runAsNonRoot: true"));
    assert!(collector.contains("- name: collector\n          image: example.invalid/sift:1.2.3"));
    assert!(collector.contains("runAsNonRoot: false\n            runAsUser: 0"));
    assert!(collector.contains("readOnlyRootFilesystem: true"));
    assert!(collector.contains("seccompProfile:"));
    assert!(collector.contains("drop: [\"ALL\"]"));
    assert!(!collector.contains("privileged: true"));
    assert!(collector.contains("requests: { cpu: 25m, memory: 64Mi }"));
    assert!(collector.contains("limits: { cpu: 500m, memory: 256Mi }"));
    assert!(!collector.contains("kind: ClusterRole"));
    assert!(!collector.contains("kind: Role"));
    assert!(!collector.contains("REPLACE_"));

    let ingest_help = sift(&["llm", "--topic", "ingest"]);
    assert!(ingest_help.contains("--cri-root /var/log/pods"));
    assert!(ingest_help.contains("sift k8s collector render"));
    assert!(ingest_help.contains("same checkpointed `axiom.service.log.v1`"));

    let operations_help = sift(&["llm", "--topic", "operations"]);
    assert!(operations_help.contains("backup --url <service> --dest <uri>"));
    assert!(operations_help.contains("explicit offline-only mode"));

    let backup_help = sift(&["backup", "--help"]);
    assert!(backup_help.contains("--url"));
    assert!(backup_help.contains("--data-dir"));
    assert!(backup_help.contains("--token"));
    let ambiguous = sift_output(&[
        "backup",
        "--url",
        "http://sift:7380",
        "--data-dir",
        "/tmp/offline-sift",
        "--dest",
        "file:///tmp/sift-backups",
    ]);
    assert!(!ambiguous.status.success());
    assert!(String::from_utf8_lossy(&ambiguous.stderr).contains("cannot be used with"));

    let missing_source = sift_output(&["backup", "--dest", "file:///tmp/sift-backups"]);
    assert!(!missing_source.status.success());
    assert!(String::from_utf8_lossy(&missing_source.stderr).contains("required"));

    let offline_data = tempfile::tempdir().expect("offline journal directory");
    let offline_backup = tempfile::tempdir().expect("offline backup directory");
    let offline_destination = format!("file://{}", offline_backup.path().display());
    let offline = sift_output(&[
        "backup",
        "--data-dir",
        offline_data.path().to_str().expect("utf-8 temp path"),
        "--dest",
        &offline_destination,
    ]);
    assert!(offline.status.success(), "offline backup: {offline:?}");
}
// </HANDWRITE>

fn parse_yaml_documents(body: &str) -> Vec<Value> {
    serde_yaml::Deserializer::from_str(body)
        .map(|document| Value::deserialize(document).expect("parse Kubernetes YAML document"))
        .collect()
}

fn object<'a>(objects: &'a [Value], kind: &str, name: &str) -> &'a Value {
    objects
        .iter()
        .find(|object| object["kind"] == kind && object["metadata"]["name"] == name)
        .unwrap_or_else(|| panic!("missing {kind}/{name}"))
}

fn sift_resource(replicas_per_shard: u32, voter_count: u32) -> Sift {
    serde_json::from_value(json!({
        "apiVersion": "sift.axiom.dev/v1alpha1",
        "kind": "Sift",
        "metadata": {
            "name": "events",
            "namespace": "observability",
            "uid": "sift-owner-uid",
            "generation": 7
        },
        "spec": {
            "image": "example.invalid/sift:gke-test",
            "replicasPerShard": replicas_per_shard,
            "voterCount": voter_count,
            "dataSize": "1Gi",
            "auth": "off",
            "backup": {
                "schedule": "*/5 * * * *",
                "destination": "gs://sift-acceptance/backups",
                "retentionSecs": 3600,
                "adminTokenSecret": "sift-backup-admin"
            }
        }
    }))
    .expect("decode Sift resource")
}

#[test]
fn operator_yaml_is_parseable_and_contains_gke_control_plane_dependencies() {
    let crd = parse_yaml_documents(&sift::deploy::crd_yaml());
    let crd = object(&crd, "CustomResourceDefinition", "sifts.sift.axiom.dev");
    let spec_schema = &crd["spec"]["versions"][0]["schema"]["openAPIV3Schema"]["properties"]
        ["spec"]["properties"];
    assert_eq!(spec_schema["replicasPerShard"]["maximum"], 1);
    assert_eq!(spec_schema["auth"]["enum"], json!(["off", "required"]));
    assert_eq!(
        spec_schema["backup"]["properties"]["adminTokenSecret"]["type"],
        "string"
    );

    let yaml = sift::deploy::operator_yaml_with_image(
        "observability-system",
        "asia-east1-docker.pkg.dev/example/axiom/sift:gke-test",
    )
    .expect("render operator YAML");
    let documents = parse_yaml_documents(&yaml);

    assert_eq!(
        object(&documents, "Namespace", "observability-system")["apiVersion"],
        "v1"
    );
    let deployment = object(&documents, "Deployment", "sift-operator");
    assert_eq!(deployment["metadata"]["namespace"], "observability-system");
    assert_eq!(
        deployment["spec"]["template"]["spec"]["containers"][0]["image"],
        "asia-east1-docker.pkg.dev/example/axiom/sift:gke-test"
    );

    let role = object(&documents, "ClusterRole", "sift-operator");
    let rules = role["rules"].as_array().expect("ClusterRole rules");
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!(["sift.axiom.dev"])
            && rule["resources"] == json!(["sifts"])
            && rule["verbs"]
                .as_array()
                .is_some_and(|verbs| verbs.contains(&json!("delete")))
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!(["sift.axiom.dev"])
            && rule["resources"]
                .as_array()
                .is_some_and(|resources| resources.contains(&json!("sifts/finalizers")))
            && rule["verbs"]
                .as_array()
                .is_some_and(|verbs| verbs.contains(&json!("update")))
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!(["coordination.k8s.io"])
            && rule["resources"] == json!(["leases"])
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!(["batch"]) && rule["resources"] == json!(["cronjobs"])
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!([""])
            && rule["resources"]
                .as_array()
                .is_some_and(|resources| resources.contains(&json!("serviceaccounts")))
    }));
    assert!(sift::deploy::operator_yaml_with_image("INVALID", "example.invalid/sift:1").is_err());
    assert!(sift::deploy::operator_yaml_with_image("sift-system", "{invalid-yaml}").is_err());
}

#[test]
fn one_by_one_render_owns_children_and_wires_protected_live_backup() {
    let sift = sift_resource(1, 1);
    let objects = sift.render();

    for child in &objects {
        assert_eq!(child["metadata"]["namespace"], "observability");
        assert_eq!(
            child["metadata"]["ownerReferences"][0]["uid"],
            "sift-owner-uid"
        );
    }

    let headless = object(&objects, "Service", "events-headless");
    assert_eq!(headless["spec"]["clusterIP"], "None");
    assert_eq!(headless["spec"]["publishNotReadyAddresses"], true);

    let client = object(&objects, "Service", "events");
    assert_eq!(client["spec"]["type"], "ClusterIP");

    let stateful_set = object(&objects, "StatefulSet", "events");
    assert_eq!(stateful_set["spec"]["replicas"], 1);
    assert_eq!(
        stateful_set["spec"]["template"]["spec"]["serviceAccountName"],
        "events"
    );
    assert_eq!(
        stateful_set["spec"]["template"]["spec"]["enableServiceLinks"],
        false
    );
    assert_eq!(
        stateful_set["spec"]["template"]["spec"]["securityContext"]["fsGroup"],
        65532
    );

    object(&objects, "ServiceAccount", "events-backup");
    let cron_job = object(&objects, "CronJob", "events-backup");
    assert!(cron_job["spec"].get("suspend").is_none());
    assert!(cron_job["metadata"].get("annotations").is_none());
    assert_eq!(
        cron_job["spec"]["jobTemplate"]["spec"]["template"]["spec"]["serviceAccountName"],
        "events-backup"
    );
    let backup_pod = &cron_job["spec"]["jobTemplate"]["spec"]["template"]["spec"];
    assert!(backup_pod.get("volumes").is_none());
    assert!(backup_pod["containers"][0].get("volumeMounts").is_none());
    let backup_args = backup_pod["containers"][0]["args"]
        .as_array()
        .expect("backup args");
    assert!(backup_args.windows(2).any(|pair| {
        pair[0] == "--url" && pair[1] == "http://events.observability.svc.cluster.local:7380"
    }));
    assert!(!backup_args.contains(&json!("--data-dir")));
    let backup_env = backup_pod["containers"][0]["env"]
        .as_array()
        .expect("backup env");
    let token = backup_env
        .iter()
        .find(|entry| entry["name"] == "SIFT_BACKUP_TOKEN")
        .expect("backup token env");
    assert_eq!(
        token["valueFrom"]["secretKeyRef"],
        json!({"name":"sift-backup-admin","key":"token"})
    );

    let status = sift.status_patch(&ReadyFacts {
        ready: HashMap::from([("events".to_string(), 1)]),
    });
    assert_eq!(status["status"]["phase"], "Ready");
    assert_eq!(status["status"]["observedGeneration"], 7);
    assert_eq!(status["status"]["desiredShardCount"], 1);
    assert_eq!(status["status"]["currentShardCount"], 1);
    assert_eq!(status["status"]["desiredReplicasPerShard"], 1);
    assert_eq!(status["status"]["currentReadyReplicasPerShard"], 1);
    assert_eq!(status["status"]["backupPhase"], "Configured");
    assert!(status["status"]["backupMessage"]
        .as_str()
        .expect("backup message")
        .contains("execution evidence"));
    assert!(!status["status"]["backupMessage"]
        .as_str()
        .unwrap()
        .contains("Passed"));

    let mut no_backup = sift_resource(1, 1);
    no_backup.spec.backup = None;
    let status = no_backup.status_patch(&ReadyFacts {
        ready: HashMap::from([("events".to_string(), 1)]),
    });
    assert_eq!(status["status"]["backupPhase"], "NotConfigured");
}

#[test]
fn unsupported_membership_is_clamped_while_live_backup_stays_pvc_free() {
    let sift = sift_resource(3, 3);
    let objects = sift.render();
    let stateful_set = object(&objects, "StatefulSet", "events");
    assert_eq!(stateful_set["spec"]["replicas"], 1);

    let env = stateful_set["spec"]["template"]["spec"]["containers"][0]["env"]
        .as_array()
        .expect("Sift env");
    let env_value = |name: &str| {
        env.iter()
            .find(|entry| entry["name"] == name)
            .and_then(|entry| entry["value"].as_str())
            .unwrap_or_else(|| panic!("missing {name}"))
    };
    assert_eq!(env_value("REPLICAS_PER_SHARD"), "1");
    assert_eq!(env_value("VOTER_COUNT"), "1");
    assert!(object(&objects, "CronJob", "events-backup")["spec"]
        .get("suspend")
        .is_none());

    let status = sift.status_patch(&ReadyFacts {
        ready: HashMap::from([("events".to_string(), 1)]),
    });
    assert_eq!(status["status"]["phase"], "UnsupportedTopology");
    assert_eq!(status["status"]["desiredReplicasPerShard"], 3);

    let legacy_zero_defaults = sift_resource(0, 0);
    let status = legacy_zero_defaults.status_patch(&ReadyFacts {
        ready: HashMap::from([("events".to_string(), 1)]),
    });
    assert_eq!(status["status"]["phase"], "Ready");
    assert_eq!(status["status"]["desiredReplicasPerShard"], 1);
}

// HANDWRITE-END
