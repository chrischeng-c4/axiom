// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-tests-k8s-object-contract-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use serde_json::Value;

use preview::{render_files, RenderInput};

fn input() -> RenderInput {
    RenderInput {
        mr: 123,
        sha: "abc123".to_string(),
        image: "us-docker.pkg.dev/acme/uat/app:abc123".to_string(),
        app: "checkout".to_string(),
        host: "uat.example.com".to_string(),
        owner: "payments-sre".to_string(),
        ttl_hours: 48,
        control_namespace: "preview-system".to_string(),
        workload_identity: "preview-runner".to_string(),
    }
}

fn object(path: &str) -> Value {
    let files = render_files(&input()).expect("render");
    let file = files.iter().find(|file| file.path == path).expect(path);
    serde_yaml::from_str(&file.contents).expect("yaml object")
}

#[test]
fn rendered_kubernetes_objects_parse_with_expected_kinds() {
    let cases = [
        ("k8s/namespace.yaml", "Namespace"),
        ("k8s/deployment.yaml", "Deployment"),
        ("k8s/service.yaml", "Service"),
        ("router/route-binding.yaml", "ConfigMap"),
    ];

    for (path, kind) in cases {
        let object = object(path);
        assert_eq!(object["kind"], kind);
        assert!(object["apiVersion"].as_str().is_some());
        assert!(object["metadata"]["name"].as_str().is_some());
    }
}

#[test]
fn service_selector_matches_deployment_pod_labels() {
    let deployment = object("k8s/deployment.yaml");
    let service = object("k8s/service.yaml");

    let selector = service["spec"]["selector"]
        .as_object()
        .expect("service selector");
    let pod_labels = deployment["spec"]["template"]["metadata"]["labels"]
        .as_object()
        .expect("pod labels");

    for (key, value) in selector {
        assert_eq!(pod_labels.get(key), Some(value), "selector key {key}");
    }
}

#[test]
fn deployment_has_sre_required_probes_and_identity() {
    let deployment = object("k8s/deployment.yaml");
    let pod_spec = &deployment["spec"]["template"]["spec"];
    let container = &pod_spec["containers"][0];

    assert_eq!(pod_spec["serviceAccountName"], "preview-runner");
    assert_eq!(container["readinessProbe"]["httpGet"]["path"], "/readyz");
    assert_eq!(container["livenessProbe"]["httpGet"]["path"], "/healthz");
}

#[test]
fn route_binding_points_to_service_not_raw_namespace_cookie() {
    let binding = object("router/route-binding.yaml");

    assert_eq!(binding["data"]["target"], "mr-123");
    assert_eq!(binding["data"]["cookie"], "uat_target");
    assert_eq!(binding["data"]["namespace"], "uat-mr-123");
    assert_eq!(binding["data"]["service"], "checkout");
}

// </HANDWRITE>
