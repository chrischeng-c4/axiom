// HANDWRITE-BEGIN gap="sift-deployment-cli-tests" tracker="1606" reason="Verify all Dockerfile and layered Kubernetes artifact commands render expected contracts."
use std::{
    collections::HashMap,
    process::{Command, Output},
};

use serde::Deserialize;
use serde_json::{json, Value};
use service_k8s::{ManagedService, ReadyFacts};
use sift::operator::{AuthMode, Sift};

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
        "sift@0.1.1",
    ]);
    assert!(release_dockerfile.contains("SIFT_VERSION=0.1.1"));
    assert!(release_dockerfile.contains("ARG TARGETARCH=amd64"));
    assert!(release_dockerfile.contains("x86_64-unknown-linux-musl"));
    assert!(release_dockerfile.contains("aarch64-unknown-linux-musl"));
    assert!(release_dockerfile.contains("gcr.io/distroless/static-debian12:nonroot"));
    assert!(release_dockerfile.contains("sha256sum -c -"));

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
    assert!(operator.contains("daemonsets"));
    assert!(operator.contains("poddisruptionbudgets"));
    assert!(operator.contains("networkpolicies"));

    let instance = sift(&["k8s", "instance", "render", "--profile", "dev"]);
    assert!(instance.contains("kind: Sift"));
    assert!(instance.contains("replicasPerShard: 3"));
    assert!(instance.contains("voterCount: 3"));
    assert!(instance.contains("gcpProjectId:"));
    assert!(instance.contains("gkeClusterName:"));
    assert!(instance.contains("gkeLocation:"));
    assert!(instance.contains("peerTlsSecret:"));
    assert!(instance.contains("auth: \"off\""));
    assert!(instance.contains("image: REPLACE_ME__SIFT_IMAGE_DIGEST"));
    assert!(instance.contains("storeSize: 50Gi"));
    assert!(instance.contains("controlSize: 5Gi"));
    assert!(instance.contains("gatewaySize: 2Gi"));
    assert!(instance.contains("querySize: 2Gi"));

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
    assert!(collector.contains("path: /var/lib/sift"));
    assert!(collector.contains("secretKeyRef:"));
    assert!(collector.contains("configMapKeyRef:"));
    assert!(collector.contains("fieldPath: spec.nodeName"));
    assert!(collector.contains("chown -R 65532:65532 /var/lib/sift/agent"));
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
    assert!(ingest_help.contains("OTLP/gRPC on port 4317"));
    assert!(ingest_help.contains("/var/lib/sift/agent"));

    let operations_help = sift(&["llm", "--topic", "operations"]);
    assert!(operations_help.contains("backup --url <service> --dest <uri>"));
    assert!(operations_help.contains("sift mcp serve --stdio"));

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
            "peerTlsSecret": "sift-peer-tls",
            "replicasPerShard": replicas_per_shard,
            "voterCount": voter_count,
            "dataSize": "1Gi",
            "auth": "kubernetes",
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
fn kubernetes_auth_renders_the_runtime_auth_delegator_binding() {
    let sift = sift_resource(3, 3);
    let objects = sift.render();
    let binding = object(
        &objects,
        "ClusterRoleBinding",
        "sift.observability.events.auth-delegator",
    );

    assert!(binding["metadata"].get("namespace").is_none());
    assert!(binding["metadata"].get("ownerReferences").is_none());
    assert_eq!(
        binding["metadata"]["labels"]["sift.axiom.dev/owner-namespace"],
        "observability"
    );
    assert_eq!(
        binding["metadata"]["labels"]["service-k8s.axiom.dev/owner-uid"],
        "sift-owner-uid"
    );
    assert_eq!(binding["roleRef"]["kind"], "ClusterRole");
    assert_eq!(binding["roleRef"]["name"], "system:auth-delegator");
    assert_eq!(
        binding["subjects"],
        json!([
            {
                "kind": "ServiceAccount",
                "name": "events",
                "namespace": "observability"
            },
            {
                "kind": "ServiceAccount",
                "name": "events-store",
                "namespace": "observability"
            }
        ])
    );

    let targets = sift.cluster_scoped_children();
    assert_eq!(targets.len(), 1);
    assert!(targets[0].desired);
    assert_eq!(targets[0].kind, "ClusterRoleBinding");
    assert_eq!(
        targets[0].expected_labels["service-k8s.axiom.dev/owner-uid"],
        "sift-owner-uid"
    );

    let mut disabled = sift.clone();
    disabled.spec.auth = AuthMode::Off;
    assert!(disabled
        .render()
        .iter()
        .all(|object| object["kind"] != "ClusterRoleBinding"));
    let targets = disabled.cluster_scoped_children();
    assert_eq!(targets.len(), 1);
    assert!(!targets[0].desired);
}

#[test]
fn role_egress_has_no_world_wide_dns_or_https_rule() {
    let sift = sift_resource(3, 3);
    let objects = sift.render();

    for policy in objects
        .iter()
        .filter(|object| object["kind"] == "NetworkPolicy")
    {
        for rule in policy["spec"]["egress"].as_array().expect("egress rules") {
            let restricted_port = rule["ports"]
                .as_array()
                .expect("egress ports")
                .iter()
                .any(|port| matches!(port["port"].as_i64(), Some(53 | 443)));
            if restricted_port {
                assert!(
                    rule["to"]
                        .as_array()
                        .expect("egress peers")
                        .iter()
                        .all(|peer| peer.as_object().is_some_and(|peer| !peer.is_empty())),
                    "{} contains a world-wide DNS/HTTPS peer: {rule}",
                    policy["metadata"]["name"]
                );
            }
        }
    }

    let backup_external = object(&objects, "FQDNNetworkPolicy", "events-backup-google-apis");
    assert_eq!(
        backup_external["spec"]["egress"][0]["matches"],
        json!([{"name":"storage.googleapis.com"}])
    );
    assert_eq!(
        backup_external["spec"]["egress"][0]["ports"],
        json!([{"protocol":"TCP", "port":443}])
    );
    assert_eq!(
        backup_external["spec"]["podSelector"]["matchLabels"],
        json!({
            "app.kubernetes.io/name": "sift",
            "app.kubernetes.io/instance": "events",
            "app.kubernetes.io/component": "backup",
            "sift.axiom.dev/role": "backup"
        })
    );

    let mut no_gcs = sift.clone();
    no_gcs.spec.backup = None;
    let prunes = no_gcs.prunes();
    assert!(prunes.iter().any(|target| {
        target.api_version == "networking.gke.io/v1alpha1"
            && target.kind == "FQDNNetworkPolicy"
            && target.name == "events-backup-google-apis"
    }));
}

#[tokio::test]
async fn reconcile_limits_kubernetes_reviews_to_the_discovered_api_endpoint() {
    let service = tower::service_fn(
        |request: axum::http::Request<kube::client::Body>| async move {
            assert_eq!(
                request.uri().path(),
                "/api/v1/namespaces/default/endpoints/kubernetes"
            );
            let body = json!({
                "apiVersion": "v1",
                "kind": "Endpoints",
                "metadata": {"name":"kubernetes", "namespace":"default"},
                "subsets": [{
                    "addresses": [{"ip":"10.20.30.40"}],
                    "ports": [{"name":"https", "port":6443, "protocol":"TCP"}]
                }]
            });
            Ok::<_, std::convert::Infallible>(
                axum::http::Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(kube::client::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
        },
    );
    let client = kube::Client::new(service, "default");
    let plan = sift_resource(3, 3)
        .reconcile_plan(client)
        .await
        .expect("discover the Kubernetes API endpoint");

    for role in ["gateway", "query", "store", "control"] {
        let policy = object(
            &plan.children,
            "NetworkPolicy",
            &format!("events-{role}-network"),
        );
        assert!(policy["spec"]["egress"]
            .as_array()
            .expect("egress rules")
            .iter()
            .any(|rule| {
                rule["ports"]
                    .as_array()
                    .is_some_and(|ports| ports.iter().any(|port| port["port"] == 6443))
                    && rule["to"].as_array().is_some_and(|peers| {
                        peers
                            .iter()
                            .any(|peer| peer["ipBlock"]["cidr"] == "10.20.30.40/32")
                    })
            }));
    }
    for role in ["agent", "backup"] {
        let policy = object(
            &plan.children,
            "NetworkPolicy",
            &format!("events-{role}-network"),
        );
        assert!(!policy.to_string().contains("10.20.30.40/32"));
    }
}

#[tokio::test]
async fn reconcile_fails_closed_when_the_kubernetes_api_has_no_ready_endpoint() {
    let service = tower::service_fn(
        |_request: axum::http::Request<kube::client::Body>| async move {
            let body = json!({
                "apiVersion": "v1",
                "kind": "Endpoints",
                "metadata": {"name":"kubernetes", "namespace":"default"},
                "subsets": []
            });
            Ok::<_, std::convert::Infallible>(
                axum::http::Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(kube::client::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
        },
    );
    let error = match sift_resource(3, 3)
        .reconcile_plan(kube::Client::new(service, "default"))
        .await
    {
        Ok(_) => panic!("Kubernetes auth must not start without a bounded API endpoint"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("has no ready addresses"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn operator_yaml_is_parseable_and_contains_gke_control_plane_dependencies() {
    let crd = parse_yaml_documents(&sift::deploy::crd_yaml());
    let crd = object(&crd, "CustomResourceDefinition", "sifts.sift.axiom.dev");
    let spec_schema = &crd["spec"]["versions"][0]["schema"]["openAPIV3Schema"]["properties"]
        ["spec"]["properties"];
    assert_eq!(spec_schema["replicasPerShard"]["minimum"], 3);
    assert_eq!(spec_schema["replicasPerShard"]["maximum"], 3);
    assert_eq!(
        spec_schema["auth"]["enum"],
        json!(["off", "required", "kubernetes"])
    );
    assert_eq!(spec_schema["peerTlsSecret"]["type"], "string");
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
            && rule["verbs"].as_array().is_some_and(|verbs| {
                verbs.contains(&json!("delete")) && verbs.contains(&json!("patch"))
            })
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!(["sift.axiom.dev"])
            && rule["resources"] == json!(["projects"])
            && rule["verbs"] == json!(["get", "create", "update"])
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
        rule["apiGroups"] == json!(["policy"])
            && rule["resources"] == json!(["poddisruptionbudgets"])
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!(["networking.k8s.io"])
            && rule["resources"] == json!(["networkpolicies"])
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!(["networking.gke.io"])
            && rule["resources"] == json!(["fqdnnetworkpolicies"])
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!(["rbac.authorization.k8s.io"])
            && rule["resources"] == json!(["roles", "rolebindings"])
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!(["rbac.authorization.k8s.io"])
            && rule["resources"] == json!(["clusterrolebindings"])
            && rule["verbs"]
                .as_array()
                .is_some_and(|verbs| verbs.contains(&json!("patch")))
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!(["rbac.authorization.k8s.io"])
            && rule["resources"] == json!(["clusterroles"])
            && rule["resourceNames"] == json!(["system:auth-delegator"])
            && rule["verbs"] == json!(["bind"])
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!([""])
            && rule["resources"]
                .as_array()
                .is_some_and(|resources| resources.contains(&json!("serviceaccounts")))
    }));
    assert!(rules.iter().any(|rule| {
        rule["apiGroups"] == json!([""])
            && rule["resources"]
                .as_array()
                .is_some_and(|resources| resources.contains(&json!("endpoints")))
    }));
    assert!(sift::deploy::operator_yaml_with_image("INVALID", "example.invalid/sift:1").is_err());
    assert!(sift::deploy::operator_yaml_with_image("sift-system", "{invalid-yaml}").is_err());
}

#[test]
fn role_topology_owns_children_and_wires_protected_live_backup() {
    let sift = sift_resource(3, 3);
    let objects = sift.render();

    for child in &objects {
        if child["kind"] == "ClusterRoleBinding" {
            continue;
        }
        assert_eq!(child["metadata"]["namespace"], "observability");
        assert_eq!(
            child["metadata"]["ownerReferences"][0]["uid"],
            "sift-owner-uid"
        );
    }

    for name in ["events-store-headless", "events-control-headless"] {
        let headless = object(&objects, "Service", name);
        assert_eq!(headless["spec"]["clusterIP"], "None");
        assert_eq!(headless["spec"]["publishNotReadyAddresses"], true);
        assert!(headless["spec"]["ports"]
            .as_array()
            .expect("headless ports")
            .iter()
            .any(|port| port["name"] == "raft-mtls" && port["port"] == 7381));
    }

    let client = object(&objects, "Service", "events");
    assert_eq!(client["spec"]["type"], "ClusterIP");
    assert_eq!(client["spec"]["selector"]["sift.axiom.dev/role"], "gateway");
    for name in ["events-store", "events-control"] {
        object(&objects, "Service", name);
    }

    for (kind, name, role) in [
        ("StatefulSet", "events-store", "store"),
        ("StatefulSet", "events-control", "control"),
        ("Deployment", "events-gateway", "gateway"),
        ("Deployment", "events-query", "query"),
        ("DaemonSet", "events-agent", "agent"),
    ] {
        let workload = object(&objects, kind, name);
        assert_eq!(
            workload["spec"]["template"]["spec"]["serviceAccountName"],
            if role == "store" {
                "events-store"
            } else {
                "events"
            }
        );
        assert_eq!(
            workload["spec"]["template"]["spec"]["enableServiceLinks"],
            false
        );
        assert_eq!(
            workload["spec"]["template"]["spec"]["automountServiceAccountToken"],
            role != "agent"
        );
        assert_eq!(
            workload["spec"]["template"]["metadata"]["labels"]["sift.axiom.dev/role"],
            role
        );
    }
    let project_role = object(&objects, "Role", "events-agent-project");
    assert_eq!(
        project_role["rules"][0],
        json!({
            "apiGroups":["sift.axiom.dev"],
            "resources":["projects"],
            "resourceNames":["events"],
            "verbs":["get", "create", "update"]
        })
    );
    let project_binding = object(&objects, "RoleBinding", "events-agent-project");
    assert_eq!(project_binding["subjects"][0]["name"], "events");
    assert_eq!(project_binding["subjects"][0]["namespace"], "observability");
    assert_eq!(project_binding["subjects"][1]["name"], "events-backup");
    assert_eq!(project_binding["subjects"][1]["namespace"], "observability");
    let agent_pod = &object(&objects, "DaemonSet", "events-agent")["spec"]["template"]["spec"];
    assert!(agent_pod["volumes"]
        .as_array()
        .expect("agent volumes")
        .iter()
        .any(|volume| {
            volume["name"] == "sift-client-token"
                && volume["projected"]["sources"][0]["serviceAccountToken"]["audience"]
                    == "sift.axiom.dev"
        }));
    let agent_env = agent_pod["containers"][0]["env"]
        .as_array()
        .expect("agent env");
    assert!(agent_env.iter().any(|entry| {
        entry["name"] == "SIFT_TOKEN_FILE" && entry["value"] == "/var/run/secrets/sift/client/token"
    }));
    assert_eq!(
        object(&objects, "StatefulSet", "events-store")["spec"]["replicas"],
        3
    );
    assert_eq!(
        object(&objects, "StatefulSet", "events-control")["spec"]["replicas"],
        3
    );
    let gateway_env = object(&objects, "Deployment", "events-gateway")["spec"]["template"]["spec"]
        ["containers"][0]["env"]
        .as_array()
        .expect("gateway env");
    for (name, value) in [
        (
            "SIFT_STORE_ENDPOINT",
            "http://events-store.observability.svc.cluster.local:7380",
        ),
        (
            "SIFT_STORE_GRPC_ENDPOINT",
            "http://events-store.observability.svc.cluster.local:4317",
        ),
        (
            "SIFT_QUERY_ENDPOINT",
            "http://events-query.observability.svc.cluster.local:7380",
        ),
    ] {
        assert!(gateway_env
            .iter()
            .any(|entry| entry["name"] == name && entry["value"] == value));
    }
    for role in ["store", "control"] {
        let pod =
            &object(&objects, "StatefulSet", &format!("events-{role}"))["spec"]["template"]["spec"];
        assert!(pod["volumes"]
            .as_array()
            .expect("pod volumes")
            .iter()
            .any(|volume| {
                volume["name"] == "peer-tls" && volume["secret"]["secretName"] == "sift-peer-tls"
            }));
        let container = &pod["containers"][0];
        assert!(container["ports"]
            .as_array()
            .expect("container ports")
            .iter()
            .any(|port| port["name"] == "raft-mtls" && port["containerPort"] == 7381));
        let env = container["env"].as_array().expect("container env");
        let expected_headless = format!("events-{role}-headless.observability.svc.cluster.local");
        assert!(env.iter().any(|entry| {
            entry["name"] == "SIFT_RAFT_HEADLESS" && entry["value"] == expected_headless
        }));
        for (name, value) in [
            ("SIFT_PEER_MTLS", "on"),
            ("SIFT_PEER_PORT", "7381"),
            (
                "SIFT_PEER_TLS_CERT",
                "/var/run/secrets/sift/peer-tls/tls.crt",
            ),
            (
                "SIFT_PEER_TLS_KEY",
                "/var/run/secrets/sift/peer-tls/tls.key",
            ),
            ("SIFT_PEER_TLS_CA", "/var/run/secrets/sift/peer-tls/ca.crt"),
        ] {
            assert!(env
                .iter()
                .any(|entry| entry["name"] == name && entry["value"] == value));
        }
    }
    let default_deny = object(&objects, "NetworkPolicy", "events-network");
    assert_eq!(default_deny["spec"]["ingress"], json!([]));
    assert_eq!(default_deny["spec"]["egress"], json!([]));
    let store_network = object(&objects, "NetworkPolicy", "events-store-network");
    assert!(store_network["spec"]["ingress"]
        .as_array()
        .expect("store ingress rules")
        .iter()
        .any(|rule| {
            rule["ports"]
                .as_array()
                .expect("store ingress ports")
                .iter()
                .any(|port| port["port"] == 7381)
                && rule["from"][0]["podSelector"]["matchLabels"]["sift.axiom.dev/role"] == "store"
        }));
    for role in ["gateway", "query", "agent", "backup"] {
        let policy = object(&objects, "NetworkPolicy", &format!("events-{role}-network"));
        assert!(
            !policy["spec"]["ingress"].to_string().contains("7381"),
            "Raft must not be reachable through the {role} role policy"
        );
    }

    let cron_job = object(&objects, "CronJob", "events-backup");
    assert!(cron_job["spec"].get("suspend").is_none());
    assert!(cron_job["metadata"].get("annotations").is_none());
    assert_eq!(
        cron_job["spec"]["jobTemplate"]["spec"]["template"]["spec"]["serviceAccountName"],
        "events-backup"
    );
    let backup_pod = &cron_job["spec"]["jobTemplate"]["spec"]["template"]["spec"];
    assert!(backup_pod["volumes"]
        .as_array()
        .expect("backup volumes")
        .iter()
        .any(|volume| volume["name"] == "sift-client-token"));
    assert!(backup_pod["containers"][0]["volumeMounts"]
        .as_array()
        .expect("backup mounts")
        .iter()
        .any(|mount| mount["name"] == "sift-client-token"));
    let backup_args = backup_pod["containers"][0]["args"]
        .as_array()
        .expect("backup args");
    assert!(backup_args.windows(2).any(|pair| {
        pair[0] == "--url" && pair[1] == "http://events.observability.svc.cluster.local:7380"
    }));
    assert!(!backup_args.contains(&json!("--data-dir")));
    assert!(backup_args.windows(2).any(|pair| {
        pair[0] == "--token-file" && pair[1] == "/var/run/secrets/sift/client/token"
    }));
    assert!(backup_args
        .windows(2)
        .any(|pair| pair[0] == "--project" && pair[1] == "events"));
    assert!(backup_pod["containers"][0]["env"]
        .as_array()
        .expect("backup env")
        .is_empty());

    let status = sift.status_patch(&ReadyFacts {
        ready: HashMap::from([
            ("events-store".to_string(), 3),
            ("events-control".to_string(), 3),
            ("events-gateway".to_string(), 1),
            ("events-query".to_string(), 1),
            ("events-agent".to_string(), 1),
        ]),
    });
    assert_eq!(status["status"]["phase"], "Ready");
    assert_eq!(status["status"]["observedGeneration"], 7);
    assert_eq!(status["status"]["desiredShardCount"], 1);
    assert_eq!(status["status"]["currentShardCount"], 1);
    assert_eq!(status["status"]["desiredReplicasPerShard"], 3);
    assert_eq!(status["status"]["currentReadyReplicasPerShard"], 3);
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
        ready: HashMap::from([
            ("events-store".to_string(), 3),
            ("events-control".to_string(), 3),
            ("events-gateway".to_string(), 1),
            ("events-query".to_string(), 1),
            ("events-agent".to_string(), 1),
        ]),
    });
    assert_eq!(status["status"]["backupPhase"], "NotConfigured");
}

#[test]
fn unsupported_membership_is_refused_in_status_without_weakening_the_rendered_quorum() {
    let sift = sift_resource(1, 1);
    let objects = sift.render();
    let stateful_set = object(&objects, "StatefulSet", "events-store");
    assert_eq!(stateful_set["spec"]["replicas"], 3);

    let env = stateful_set["spec"]["template"]["spec"]["containers"][0]["env"]
        .as_array()
        .expect("Sift env");
    let env_value = |name: &str| {
        env.iter()
            .find(|entry| entry["name"] == name)
            .and_then(|entry| entry["value"].as_str())
            .unwrap_or_else(|| panic!("missing {name}"))
    };
    assert_eq!(env_value("REPLICAS_PER_SHARD"), "3");
    assert_eq!(env_value("VOTER_COUNT"), "3");
    assert!(object(&objects, "CronJob", "events-backup")["spec"]
        .get("suspend")
        .is_none());

    let status = sift.status_patch(&ReadyFacts {
        ready: HashMap::from([("events-store".to_string(), 3)]),
    });
    assert_eq!(status["status"]["phase"], "UnsupportedTopology");
    assert_eq!(status["status"]["desiredReplicasPerShard"], 1);

    let legacy_zero_defaults = sift_resource(0, 0);
    let status = legacy_zero_defaults.status_patch(&ReadyFacts {
        ready: HashMap::new(),
    });
    assert_eq!(status["status"]["phase"], "UnsupportedTopology");
    assert_eq!(status["status"]["desiredReplicasPerShard"], 0);
}

#[test]
fn mvp_crd_and_operator_render_archive_restore_limits_and_role_storage() {
    let crd = parse_yaml_documents(&sift::deploy::crd_yaml());
    let crd = object(&crd, "CustomResourceDefinition", "sifts.sift.axiom.dev");
    let spec_schema = &crd["spec"]["versions"][0]["schema"]["openAPIV3Schema"]["properties"]
        ["spec"]["properties"];
    assert_eq!(
        spec_schema["archive"]["properties"]["destination"]["type"],
        "string"
    );
    assert_eq!(
        spec_schema["bootstrap"]["properties"]["archiveManifestUri"]["type"],
        "string"
    );
    for field in ["storeSize", "controlSize", "gatewaySize", "querySize"] {
        assert_eq!(
            spec_schema["storage"]["properties"][field]["type"],
            "string"
        );
    }
    assert_eq!(
        spec_schema["ingest"]["properties"]["maxItemsPerMinute"]["minimum"],
        1
    );
    assert_eq!(
        spec_schema["ingest"]["properties"]["maxConcurrentRequests"]["minimum"],
        1
    );
    assert_eq!(
        spec_schema["placement"]["properties"]["nodeSelector"]["type"],
        "object"
    );

    let sift: Sift = serde_json::from_value(json!({
        "apiVersion": "sift.axiom.dev/v1alpha1",
        "kind": "Sift",
        "metadata": {
            "name": "mvp",
            "namespace": "observability",
            "uid": "mvp-owner-uid",
            "generation": 11
        },
        "spec": {
            "image": "example.invalid/sift@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "peerTlsSecret": "sift-peer-tls",
            "replicasPerShard": 3,
            "voterCount": 3,
            "auth": "kubernetes",
            "archive": {"destination": "gs://sift-mvp/archive"},
            "bootstrap": {"archiveManifestUri": "gs://sift-mvp/archive/manifest.json"},
            "storage": {
                "storeSize": "51Gi",
                "controlSize": "6Gi",
                "gatewaySize": "3Gi",
                "querySize": "4Gi"
            },
            "ingest": {
                "maxItemsPerMinute": 720000,
                "maxConcurrentRequests": 32
            },
            "placement": {"nodeSelector": {"axiom-run-id": "mvp-run"}},
            "gcpProjectId": "example",
            "gkeClusterName": "shared",
            "gkeLocation": "asia-east1"
        }
    }))
    .expect("decode MVP Sift resource");
    let objects = sift.render();

    assert_eq!(
        object(&objects, "StatefulSet", "mvp-store")["spec"]["volumeClaimTemplates"][0]["spec"]
            ["resources"]["requests"]["storage"],
        "51Gi"
    );
    assert_eq!(
        object(&objects, "StatefulSet", "mvp-control")["spec"]["volumeClaimTemplates"][0]["spec"]
            ["resources"]["requests"]["storage"],
        "6Gi"
    );
    assert_eq!(
        object(&objects, "PersistentVolumeClaim", "mvp-gateway-data")["spec"]["resources"]
            ["requests"]["storage"],
        "3Gi"
    );
    assert_eq!(
        object(&objects, "PersistentVolumeClaim", "mvp-query-data")["spec"]["resources"]
            ["requests"]["storage"],
        "4Gi"
    );

    object(&objects, "ServiceAccount", "mvp-store");
    object(&objects, "ServiceAccount", "mvp-backup");
    assert_eq!(
        object(&objects, "StatefulSet", "mvp-store")["spec"]["template"]["spec"]
            ["serviceAccountName"],
        "mvp-store"
    );
    for (kind, name) in [
        ("StatefulSet", "mvp-store"),
        ("StatefulSet", "mvp-control"),
        ("Deployment", "mvp-gateway"),
        ("Deployment", "mvp-query"),
        ("DaemonSet", "mvp-agent"),
    ] {
        assert_eq!(
            object(&objects, kind, name)["spec"]["template"]["spec"]["nodeSelector"]
                ["axiom-run-id"],
            "mvp-run"
        );
    }

    let store_env = object(&objects, "StatefulSet", "mvp-store")["spec"]["template"]["spec"]
        ["containers"][0]["env"]
        .as_array()
        .expect("store env");
    for (name, value) in [
        ("SIFT_ARCHIVE_DESTINATION", "gs://sift-mvp/archive"),
        (
            "SIFT_BOOTSTRAP_ARCHIVE_MANIFEST_URI",
            "gs://sift-mvp/archive/manifest.json",
        ),
        ("SIFT_MAX_INGEST_ITEMS_PER_PROJECT_WINDOW", "720000"),
        ("SIFT_MAX_CONCURRENT_INGEST_PER_PROJECT", "32"),
        ("SIFT_MAX_EVENTS_PER_BATCH", "1000"),
    ] {
        assert!(store_env
            .iter()
            .any(|entry| entry["name"] == name && entry["value"] == value));
    }

    let status = sift.status_patch(&ReadyFacts {
        ready: HashMap::from([
            ("mvp-store".to_string(), 3),
            ("mvp-control".to_string(), 3),
            ("mvp-gateway".to_string(), 1),
            ("mvp-query".to_string(), 1),
            ("mvp-agent".to_string(), 3),
        ]),
    });
    assert_eq!(status["status"]["archivePhase"], "Configured");
    assert_eq!(status["status"]["archiveWatermark"], 0);
    assert_eq!(status["status"]["lastArchiveManifest"], "");
    assert_eq!(status["status"]["restorePhase"], "Restored");
    assert_eq!(
        status["status"]["restoreSourceManifest"],
        "gs://sift-mvp/archive/manifest.json"
    );
    assert_eq!(status["status"]["backpressure"], "Healthy");
    assert_eq!(status["status"]["lastDataError"], "");

    let restoring = sift.status_patch(&ReadyFacts {
        ready: HashMap::from([
            ("mvp-store".to_string(), 1),
            ("mvp-control".to_string(), 3),
            ("mvp-gateway".to_string(), 1),
            ("mvp-query".to_string(), 1),
            ("mvp-agent".to_string(), 3),
        ]),
    });
    assert_eq!(restoring["status"]["restorePhase"], "Restoring");
}

// HANDWRITE-END
