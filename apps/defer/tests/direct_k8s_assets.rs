// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-direct-k8s" tracker="#766" reason="Offline contract for Defer direct Kustomize singleton and operator-owned HA boundary."
use serde_yaml::Value;

fn yaml(input: &str) -> Value {
    serde_yaml::from_str(input).expect("manifest parses")
}

fn yaml_documents(input: &str) -> Vec<Value> {
    input
        .split("\n---\n")
        .filter(|document| !document.trim().is_empty())
        .map(yaml)
        .collect()
}

fn named<'a>(objects: &'a [Value], kind: &str, name: &str) -> &'a Value {
    objects
        .iter()
        .find(|object| object["kind"] == kind && object["metadata"]["name"] == name)
        .unwrap_or_else(|| panic!("missing {kind}/{name}"))
}

fn render_kustomize(profile: &str) -> Vec<Value> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(profile);
    let output = std::process::Command::new("kubectl")
        .arg("kustomize")
        .arg(&path)
        .output()
        .unwrap_or_else(|error| {
            panic!("kubectl is required to render {}: {error}", path.display())
        });
    assert!(
        output.status.success(),
        "{} must render: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    yaml_documents(&String::from_utf8(output.stdout).unwrap())
}

fn assert_connected_services_and_pdb(resources: &[Value]) {
    let selector = yaml("{app: defer, role: server}");
    let headless = named(resources, "Service", "defer-headless");
    assert_eq!(headless["spec"]["clusterIP"], "None");
    assert_eq!(headless["spec"]["publishNotReadyAddresses"], true);
    assert_eq!(&headless["spec"]["selector"], &selector);
    assert_eq!(
        headless["spec"]["ports"],
        yaml("[{name: http, port: 7141, targetPort: http, protocol: TCP}, {name: raft, port: 7142, targetPort: raft, protocol: TCP}]")
    );
    let client = named(resources, "Service", "defer");
    assert_eq!(&client["spec"]["selector"], &selector);
    assert_eq!(
        client["spec"]["ports"],
        yaml("[{name: http, port: 7141, targetPort: http, protocol: TCP}]")
    );
    let pdb = named(resources, "PodDisruptionBudget", "defer");
    assert_eq!(pdb["spec"]["maxUnavailable"], 0);
    assert_eq!(&pdb["spec"]["selector"]["matchLabels"], &selector);
}

#[test]
fn direct_base_is_a_restricted_durable_singleton() {
    let statefulset = yaml(include_str!("../k8s/base/statefulset.yaml"));
    assert_eq!(statefulset["kind"], "StatefulSet");
    assert_eq!(statefulset["spec"]["replicas"], 1);
    assert_eq!(statefulset["spec"]["serviceName"], "defer-headless");
    assert_eq!(
        statefulset["spec"]["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"],
        "10Gi"
    );
    let container = &statefulset["spec"]["template"]["spec"]["containers"][0];
    let pod_security = &statefulset["spec"]["template"]["spec"]["securityContext"];
    assert_eq!(pod_security["runAsNonRoot"], true);
    assert_eq!(pod_security["runAsUser"], 65532);
    assert_eq!(pod_security["runAsGroup"], 65532);
    assert_eq!(pod_security["seccompProfile"]["type"], "RuntimeDefault");
    assert_eq!(container["command"][0], "defer");
    assert_eq!(container["command"][1], "serve");
    assert_eq!(container["ports"][0]["containerPort"], 7141);
    assert_eq!(container["ports"][1]["containerPort"], 7142);
    assert_eq!(container["readinessProbe"]["httpGet"]["path"], "/readyz");
    assert_eq!(container["readinessProbe"]["httpGet"]["port"], "http");
    assert_eq!(container["livenessProbe"]["httpGet"]["path"], "/healthz");
    assert_eq!(container["livenessProbe"]["httpGet"]["port"], "http");
    assert_eq!(container["startupProbe"]["httpGet"]["path"], "/healthz");
    assert_eq!(container["startupProbe"]["failureThreshold"], 120);
    assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);
    assert_eq!(
        container["securityContext"]["allowPrivilegeEscalation"],
        false
    );
    assert_eq!(
        container["securityContext"]["capabilities"]["drop"][0],
        "ALL"
    );
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

    let data_mounts = container["volumeMounts"]
        .as_sequence()
        .unwrap()
        .iter()
        .filter(|mount| mount["name"] == "data")
        .collect::<Vec<_>>();
    assert_eq!(data_mounts.len(), 1);
    assert_eq!(data_mounts[0]["mountPath"], "/data");
    let claim = &statefulset["spec"]["volumeClaimTemplates"][0];
    assert_eq!(claim["metadata"]["name"], "data");
    assert_eq!(claim["spec"]["accessModes"], yaml("[ReadWriteOnce]"));

    let services = yaml_documents(include_str!("../k8s/base/service.yaml"));
    assert_eq!(services.len(), 2);
    let selector = yaml("{app: defer, role: server}");
    let headless = named(&services, "Service", "defer-headless");
    assert_eq!(headless["spec"]["clusterIP"], "None");
    assert_eq!(headless["spec"]["publishNotReadyAddresses"], true);
    assert_eq!(&headless["spec"]["selector"], &selector);
    assert_eq!(
        headless["spec"]["ports"],
        yaml("[{name: http, port: 7141, targetPort: http, protocol: TCP}, {name: raft, port: 7142, targetPort: raft, protocol: TCP}]")
    );
    let client = named(&services, "Service", "defer");
    assert_eq!(&client["spec"]["selector"], &selector);
    assert_eq!(
        client["spec"]["ports"],
        yaml("[{name: http, port: 7141, targetPort: http, protocol: TCP}]")
    );

    let pdb = yaml(include_str!("../k8s/base/pdb.yaml"));
    assert_eq!(pdb["kind"], "PodDisruptionBudget");
    assert_eq!(pdb["spec"]["maxUnavailable"], 0);
    assert_eq!(&pdb["spec"]["selector"]["matchLabels"], &selector);
}

#[test]
fn prod_profile_uses_security_components_without_voter_hpa() {
    let prod = include_str!("../k8s/overlays/prod/kustomization.yaml");
    assert!(prod.contains("DEFER_TOKEN_REGISTRY_FILE"));
    assert!(prod.contains("secretName: defer-token-registry"));
    assert!(prod.contains("readOnly: true"));
    assert!(prod.contains("../../components/observability"));
    assert!(prod.contains("../../components/network-policy"));
    assert!(!prod.contains("HorizontalPodAutoscaler"));
    assert!(!prod.contains("kind: HPA"));

    let policy = yaml(include_str!(
        "../k8s/components/network-policy/networkpolicy.yaml"
    ));
    assert_eq!(policy["kind"], "NetworkPolicy");
    assert_eq!(policy["spec"]["podSelector"]["matchLabels"]["app"], "defer");
    assert_eq!(
        policy["spec"]["podSelector"]["matchLabels"]["role"],
        "server"
    );
    let policy_types = policy["spec"]["policyTypes"].as_sequence().unwrap();
    assert!(policy_types.iter().any(|value| value == "Ingress"));
    assert!(policy_types.iter().any(|value| value == "Egress"));
    let policy_yaml = serde_yaml::to_string(&policy).unwrap();
    assert!(policy_yaml.contains("port: 7141"));
    assert!(policy_yaml.contains("port: 7142"));
}

// <HANDWRITE gap="missing-generator:e2e-test:defer-rendered-prod-security" tracker="#2215" reason="Render the composed production overlay so disconnected or invalid security patches cannot pass raw-string checks.">
#[test]
fn prod_profile_renders_the_connected_security_boundary() {
    let base = render_kustomize("k8s/base");
    assert_connected_services_and_pdb(&base);
    let resources = render_kustomize("k8s/overlays/prod");
    assert!(!resources.is_empty());
    assert_connected_services_and_pdb(&resources);
    assert!(!resources
        .iter()
        .any(|resource| resource["kind"] == "HorizontalPodAutoscaler"));
    assert!(resources
        .iter()
        .any(|resource| resource["kind"] == "ServiceMonitor"));

    let statefulset = resources
        .iter()
        .find(|resource| {
            resource["kind"] == "StatefulSet" && resource["metadata"]["name"] == "defer"
        })
        .expect("rendered Defer StatefulSet");
    let container = &statefulset["spec"]["template"]["spec"]["containers"][0];
    let pod_security = &statefulset["spec"]["template"]["spec"]["securityContext"];
    assert_eq!(pod_security["runAsNonRoot"], true);
    assert_eq!(pod_security["runAsUser"], 65532);
    assert_eq!(pod_security["runAsGroup"], 65532);
    assert_eq!(pod_security["seccompProfile"]["type"], "RuntimeDefault");
    assert_eq!(container["securityContext"]["runAsNonRoot"], true);
    assert_eq!(
        container["securityContext"]["allowPrivilegeEscalation"],
        false
    );
    assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);
    assert_eq!(
        container["securityContext"]["capabilities"]["drop"][0],
        "ALL"
    );
    let rendered_container = serde_yaml::to_string(container).unwrap();
    let rendered_statefulset = serde_yaml::to_string(statefulset).unwrap();
    assert!(rendered_container.contains("DEFER_TOKEN_REGISTRY_FILE"));
    assert!(rendered_statefulset.contains("secretName: defer-token-registry"));
    assert!(rendered_container.contains("readOnly: true"));
    assert_eq!(
        statefulset["spec"]["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"],
        "100Gi"
    );

    let policy = resources
        .iter()
        .find(|resource| resource["kind"] == "NetworkPolicy")
        .expect("rendered production NetworkPolicy");
    assert_eq!(policy["spec"]["podSelector"]["matchLabels"]["app"], "defer");
    assert_eq!(
        policy["spec"]["podSelector"]["matchLabels"]["role"],
        "server"
    );
    let rendered_policy = serde_yaml::to_string(policy).unwrap();
    assert!(rendered_policy.contains("port: 7141"));
    assert!(rendered_policy.contains("port: 7142"));
}
// </HANDWRITE>

#[test]
fn operator_deployment_uses_a_restricted_security_context() {
    let operator = yaml(include_str!("../k8s/operator/deployment.yaml"));
    let pod_security = &operator["spec"]["template"]["spec"]["securityContext"];
    assert_eq!(pod_security["runAsNonRoot"], true);
    assert_eq!(pod_security["seccompProfile"]["type"], "RuntimeDefault");
    let container = &operator["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);
    assert_eq!(
        container["securityContext"]["allowPrivilegeEscalation"],
        false
    );
    assert_eq!(
        container["securityContext"]["capabilities"]["drop"][0],
        "ALL"
    );
}
// HANDWRITE-END
