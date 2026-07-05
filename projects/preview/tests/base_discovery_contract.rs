// <HANDWRITE gap="issue-1108:base-workload-discovery" tracker="projects-preview-tests-base-discovery-contract-rs" reason="Base workload discovery normalization tests are hand-authored until Kubernetes fixture generation exists for Preview.">
use preview::{normalize_base_workload, render_files, RenderInput};
use serde_json::{json, Value};

fn deployment() -> Value {
    json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": "checkout",
            "namespace": "uat-base",
            "uid": "runtime-uid",
            "resourceVersion": "123",
            "generation": 7,
            "managedFields": [{"manager": "kubectl"}],
            "ownerReferences": [{"name": "base-owner"}]
        },
        "spec": {
            "selector": {
                "matchLabels": {
                    "app.kubernetes.io/name": "checkout",
                    "tier": "web"
                }
            },
            "template": {
                "metadata": {
                    "labels": {
                        "app.kubernetes.io/name": "checkout",
                        "tier": "web",
                        "stable": "true"
                    }
                },
                "spec": {
                    "containers": [{
                        "name": "checkout",
                        "image": "registry.local/checkout:base",
                        "ports": [{"name": "http", "containerPort": 8080}],
                        "env": [
                            {"name": "APP_MODE", "value": "uat"},
                            {"name": "SECRET_TOKEN", "valueFrom": {"secretKeyRef": {"name": "base-secret", "key": "token"}}}
                        ],
                        "resources": {
                            "requests": {"cpu": "200m", "memory": "256Mi"},
                            "limits": {"cpu": "500m", "memory": "512Mi"}
                        },
                        "readinessProbe": {"httpGet": {"path": "/readyz", "port": 8080}},
                        "livenessProbe": {"httpGet": {"path": "/healthz", "port": 8080}}
                    }]
                }
            }
        },
        "status": {"availableReplicas": 1}
    })
}

fn service() -> Value {
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": "checkout",
            "namespace": "uat-base",
            "uid": "runtime-service-uid",
            "resourceVersion": "456"
        },
        "spec": {
            "clusterIP": "10.0.0.1",
            "clusterIPs": ["10.0.0.1"],
            "selector": {
                "app.kubernetes.io/name": "checkout",
                "tier": "web"
            },
            "ports": [{
                "name": "http",
                "port": 80,
                "targetPort": 8080,
                "nodePort": 30080
            }]
        },
        "status": {"loadBalancer": {"ingress": [{"ip": "1.2.3.4"}]}}
    })
}

#[test]
fn normalizes_base_deployment_and_service_without_runtime_identity() {
    let contract = normalize_base_workload(&deployment(), &service(), "uat-base", "checkout")
        .expect("normalize base");

    assert_eq!(contract.namespace, "uat-base");
    assert_eq!(contract.deployment, "checkout");
    assert_eq!(contract.service, "checkout");
    assert_eq!(
        contract.selector.get("app.kubernetes.io/name"),
        Some(&"checkout".to_string())
    );
    assert_eq!(contract.container.name, "checkout");
    assert_eq!(contract.container.ports[0].container_port, 8080);
    assert_eq!(contract.container.env[0].name, "APP_MODE");
    assert_eq!(
        contract.container.env[1].value_from_kind,
        Some("secretKeyRef".to_string())
    );
    assert_eq!(
        contract.container.readiness_path,
        Some("/readyz".to_string())
    );
    assert_eq!(contract.service_ports[0].target_port, "8080");

    let serialized = serde_json::to_string(&contract).expect("serialize contract");
    assert!(!serialized.contains("runtime-uid"));
    assert!(!serialized.contains("10.0.0.1"));
    assert!(!serialized.contains("30080"));
    assert!(!serialized.contains("1.2.3.4"));
    assert!(contract
        .excluded_runtime_fields
        .contains(&"metadata.resourceVersion".to_string()));
    assert!(contract
        .excluded_runtime_fields
        .contains(&"secrets by default".to_string()));
}

#[test]
fn render_clone_plan_can_embed_discovered_base_contract() {
    let contract =
        normalize_base_workload(&deployment(), &service(), "uat-base", "checkout").unwrap();
    let files = render_files(&RenderInput {
        mr: 123,
        sha: "abc123".to_string(),
        image: "registry.local/checkout:abc123".to_string(),
        app: "checkout".to_string(),
        host: "uat.example.com".to_string(),
        base_namespace: "uat-base".to_string(),
        owner: "payments-sre".to_string(),
        ttl_hours: 2,
        control_namespace: "preview-system".to_string(),
        workload_identity: "preview-runner".to_string(),
        base_contract: Some(contract),
        data: None,
    })
    .expect("render files");
    let clone_plan = files
        .iter()
        .find(|file| file.path == "plans/workload-clone.json")
        .expect("clone plan");
    let clone_plan: Value = serde_json::from_str(&clone_plan.contents).expect("clone plan json");

    assert_eq!(
        clone_plan["discoveredBase"]["container"]["resources"]["requests"]["cpu"],
        "200m"
    );
    assert_eq!(
        clone_plan["discoveredBase"]["servicePorts"][0]["targetPort"],
        "8080"
    );
}

#[test]
fn refuses_ambiguous_deployment_containers_without_matching_app_name() {
    let mut deployment = deployment();
    deployment["spec"]["template"]["spec"]["containers"] = json!([
        {"name": "api", "image": "registry.local/api:base"},
        {"name": "worker", "image": "registry.local/worker:base"}
    ]);

    let err = normalize_base_workload(&deployment, &service(), "uat-base", "checkout")
        .expect_err("ambiguous containers should fail");
    assert!(err
        .to_string()
        .contains("ambiguous base Deployment containers"));
}

// </HANDWRITE>
