// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-tests-local-cicd-contract-rs" reason="Local CI/CD lifecycle smoke is hand-authored until workflow generator primitives cover binary-driven CI simulations.">
use std::process::Command;

use serde_json::{json, Value};

fn preview_bin() -> &'static str {
    env!("CARGO_BIN_EXE_preview")
}

#[test]
fn local_ci_open_update_comment_and_close_lifecycle_is_deterministic() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("preview");

    preview_render(&out, "abc123");
    assert!(out.join("spec/preview-environment.yaml").is_file());
    assert!(out.join("k8s/namespace.yaml").is_file());
    assert!(out.join("k8s/service-account.yaml").is_file());
    assert!(out.join("k8s/resource-quota.yaml").is_file());
    assert!(out.join("k8s/limit-range.yaml").is_file());
    assert!(out.join("k8s/workload-role.yaml").is_file());
    assert!(out.join("k8s/workload-role-binding.yaml").is_file());
    assert!(out.join("k8s/deployment.yaml").is_file());
    assert!(out.join("k8s/service.yaml").is_file());
    assert!(out.join("router/route-binding.yaml").is_file());
    assert!(out.join("mr-comment.md").is_file());
    assert!(out.join("cleanup-plan.json").is_file());

    preview_render(&out, "def456");
    let route_binding =
        std::fs::read_to_string(out.join("router/route-binding.yaml")).expect("route binding");
    assert!(route_binding.contains("target: mr-321"));
    assert!(route_binding.contains("namespace: uat-mr-321"));
    assert!(route_binding.contains("baseNamespace: uat-base"));
    assert!(route_binding.contains("sha: def456"));

    let clone_plan =
        std::fs::read_to_string(out.join("plans/workload-clone.json")).expect("clone plan");
    assert!(clone_plan.contains(r#""namespace": "uat-base""#));
    assert!(clone_plan.contains(r#""routeTarget": "mr-321""#));

    let comment = command_stdout(Command::new(preview_bin()).args([
        "comment",
        "--mr",
        "321",
        "--sha",
        "def456",
        "--image",
        "registry.local/checkout:def456",
        "--app",
        "checkout",
        "--host",
        "uat.local.test",
        "--base-namespace",
        "uat-base",
        "--owner",
        "payments-sre",
    ]));
    assert!(comment.contains("Route target: `mr-321`"));
    assert!(comment.contains("Base namespace: `uat-base`"));
    assert!(comment.contains("https://uat.local.test/_preview/mr-321"));
    assert!(comment.contains("X-UAT-Target: mr-321"));

    let cleanup = command_stdout(Command::new(preview_bin()).args([
        "cleanup-plan",
        "--mr",
        "321",
        "--closed",
        "--app",
        "checkout",
        "--sha",
        "def456",
        "--image",
        "registry.local/checkout:def456",
        "--host",
        "uat.local.test",
        "--base-namespace",
        "uat-base",
    ]));
    let cleanup: Value = serde_json::from_str(&cleanup).expect("cleanup json");
    assert_eq!(cleanup["action"], "delete");
    assert_eq!(cleanup["namespace"], "uat-mr-321");
    assert_eq!(cleanup["routeTarget"], "mr-321");
    assert_eq!(cleanup["deleteNamespace"], true);
    assert_eq!(cleanup["deleteRouteBinding"], true);
    assert_eq!(cleanup["protectedNamespaces"][0], "uat-base");
    assert_eq!(cleanup["protectedNamespaces"][1], "preview-system");
}

fn preview_render(out: &std::path::Path, sha: &str) {
    command_stdout(Command::new(preview_bin()).args([
        "render",
        "--mr",
        "321",
        "--sha",
        sha,
        "--image",
        &format!("registry.local/checkout:{sha}"),
        "--app",
        "checkout",
        "--host",
        "uat.local.test",
        "--base-namespace",
        "uat-base",
        "--owner",
        "payments-sre",
        "--out",
        out.to_str().expect("out path"),
    ]));
}

#[test]
fn local_ci_render_consumes_discovered_base_contract_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let contract = dir.path().join("base-contract.json");
    let out = dir.path().join("preview");
    std::fs::write(
        &contract,
        serde_json::to_string_pretty(&json!({
            "namespace": "uat-base",
            "app": "checkout",
            "deployment": "checkout",
            "service": "checkout",
            "selector": {"app.kubernetes.io/name": "checkout"},
            "podLabels": {"app.kubernetes.io/name": "checkout", "tier": "web"},
            "container": {
                "name": "checkout",
                "image": "registry.local/checkout:base",
                "ports": [{"name": "http", "containerPort": 8080}],
                "env": [{"name": "APP_MODE", "value": "uat", "valueFromKind": null}],
                "resources": {"requests": {"cpu": "200m", "memory": "256Mi"}},
                "readinessPath": "/readyz",
                "livenessPath": "/healthz"
            },
            "servicePorts": [{"name": "http", "port": 80, "targetPort": "8080"}],
            "excludedRuntimeFields": ["metadata.uid", "status", "spec.clusterIP", "secrets by default"]
        }))
        .expect("serialize base contract"),
    )
    .expect("write base contract");

    command_stdout(
        Command::new(preview_bin())
            .args([
                "render",
                "--mr",
                "654",
                "--sha",
                "abc654",
                "--image",
                "registry.local/checkout:abc654",
                "--app",
                "checkout",
                "--host",
                "uat.local.test",
                "--base-contract",
            ])
            .arg(&contract)
            .args(["--out"])
            .arg(&out),
    );

    let clone_plan =
        std::fs::read_to_string(out.join("plans/workload-clone.json")).expect("clone plan");
    let clone_plan: Value = serde_json::from_str(&clone_plan).expect("clone plan json");
    assert_eq!(
        clone_plan["discoveredBase"]["servicePorts"][0]["targetPort"],
        "8080"
    );
    assert_eq!(
        clone_plan["discoveredBase"]["container"]["resources"]["requests"]["cpu"],
        "200m"
    );
}

fn command_stdout(command: &mut Command) -> String {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("command failed to start: {err}"));
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

// </HANDWRITE>
