// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-tests-local-cicd-contract-rs" reason="Local CI/CD lifecycle smoke is hand-authored until workflow generator primitives cover binary-driven CI simulations.">
use std::process::Command;

use preview::manifest_inventory_from_dir;
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
    assert!(out.join("plans/manifest-inventory.json").is_file());
    assert!(out.join("mr-comment.md").is_file());
    assert!(out.join("cleanup-plan.json").is_file());

    let inventory = manifest_inventory_from_dir(&out).expect("manifest inventory");
    let ordered_paths: Vec<_> = inventory
        .entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    assert_eq!(
        ordered_paths,
        vec![
            "k8s/namespace.yaml",
            "k8s/service-account.yaml",
            "k8s/resource-quota.yaml",
            "k8s/limit-range.yaml",
            "k8s/workload-role.yaml",
            "k8s/workload-role-binding.yaml",
            "k8s/deployment.yaml",
            "k8s/service.yaml",
            "router/route-binding.yaml",
        ]
    );

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
fn local_apply_plan_and_gitops_bundle_are_deterministic() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("preview");
    let gitops = dir.path().join("gitops");
    preview_render(&out, "abc123");

    let apply_summary = command_stdout(Command::new(preview_bin()).args([
        "apply",
        "--dir",
        out.to_str().expect("out path"),
        "--context",
        "kind-preview-ec",
        "--plan-only",
    ]));
    assert!(apply_summary.contains("Mode: `plan-only`"));
    assert!(apply_summary.contains("Context: `kind-preview-ec`"));
    assert!(apply_summary.contains("- 00 `Namespace` `<cluster>/uat-mr-321`"));
    assert!(apply_summary.contains("- 08 `ConfigMap` `preview-system/routebinding-mr-321`"));

    command_stdout(Command::new(preview_bin()).args([
        "gitops",
        "render",
        "--dir",
        out.to_str().expect("out path"),
        "--out",
        gitops.to_str().expect("gitops path"),
    ]));

    assert!(gitops.join("manifest-inventory.json").is_file());
    assert!(gitops.join("kustomization.yaml").is_file());
    assert!(gitops.join("manifests/00-namespace.yaml").is_file());
    assert!(gitops.join("manifests/08-route-binding.yaml").is_file());

    let kustomization =
        std::fs::read_to_string(gitops.join("kustomization.yaml")).expect("kustomization");
    assert!(kustomization.contains("- manifests/00-namespace.yaml"));
    assert!(kustomization.contains("- manifests/08-route-binding.yaml"));

    let bundle_inventory =
        std::fs::read_to_string(gitops.join("manifest-inventory.json")).expect("inventory");
    assert!(bundle_inventory.contains(r#""path": "k8s/namespace.yaml""#));
    assert!(
        !bundle_inventory.contains(dir.path().to_str().expect("tempdir path")),
        "GitOps inventory leaked a local absolute path"
    );
}

#[test]
fn local_router_resolve_proves_base_preview_and_fail_closed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("preview");
    preview_render(&out, "abc123");

    let base = command_stdout(Command::new(preview_bin()).args([
        "router",
        "resolve",
        "--dir",
        out.to_str().expect("out path"),
        "--host",
        "uat.local.test",
        "--base-namespace",
        "uat-base",
        "--base-service",
        "checkout",
    ]));
    let base: Value = serde_json::from_str(&base).expect("base decision");
    assert_eq!(base["outcome"], "base");
    assert_eq!(base["namespace"], "uat-base");
    assert!(base["reason"]
        .as_str()
        .expect("reason")
        .contains("base route"));

    let preview = command_stdout(Command::new(preview_bin()).args([
        "router",
        "resolve",
        "--dir",
        out.to_str().expect("out path"),
        "--host",
        "uat.local.test",
        "--header-target",
        "mr-321",
        "--cookie-target",
        "mr-999",
    ]));
    let preview: Value = serde_json::from_str(&preview).expect("preview decision");
    assert_eq!(preview["outcome"], "preview");
    assert_eq!(preview["target"], "mr-321");
    assert_eq!(preview["namespace"], "uat-mr-321");
    assert_eq!(preview["reason"], "matched X-UAT-Target header");

    let invalid = command_stdout(Command::new(preview_bin()).args([
        "router",
        "resolve",
        "--dir",
        out.to_str().expect("out path"),
        "--host",
        "uat.local.test",
        "--cookie-target",
        "mr-999",
    ]));
    let invalid: Value = serde_json::from_str(&invalid).expect("invalid decision");
    assert_eq!(invalid["outcome"], "not-found");
    assert_eq!(invalid["target"], "mr-999");
    assert_eq!(invalid["namespace"], Value::Null);
    assert!(invalid["reason"]
        .as_str()
        .expect("reason")
        .contains("unknown route target"));
}

#[test]
fn local_cleanup_janitor_plan_reports_guarded_actions() {
    let delete = command_stdout(Command::new(preview_bin()).args([
        "cleanup",
        "plan",
        "--mr",
        "321",
        "--closed",
        "--namespace-exists",
        "--route-binding-exists",
        "--base-namespace",
        "uat-base",
        "--control-namespace",
        "preview-system",
    ]));
    let delete: Value = serde_json::from_str(&delete).expect("delete plan");
    assert_eq!(delete["action"], "delete");
    assert_eq!(delete["deleteNamespace"], true);
    assert_eq!(delete["deleteRouteBinding"], true);
    assert_eq!(delete["reason"], "MR is closed or merged");

    let protected = command_stdout(Command::new(preview_bin()).args([
        "cleanup",
        "plan",
        "--mr",
        "321",
        "--namespace",
        "uat-base",
        "--closed",
        "--namespace-exists",
        "--route-binding-exists",
        "--base-namespace",
        "uat-base",
    ]));
    let protected: Value = serde_json::from_str(&protected).expect("protected plan");
    assert_eq!(protected["action"], "keep");
    assert_eq!(protected["deleteNamespace"], false);
    assert!(protected["skipped"][0]
        .as_str()
        .expect("skip")
        .contains("protected namespace"));
}

#[test]
fn ci_templates_document_required_variables_and_command_order() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in [
        "docs/ci-templates/github-actions-preview.yaml",
        "docs/ci-templates/gitlab-ci-preview.yml",
        "docs/ci-templates/local-kind-lifecycle.sh",
    ] {
        let contents = std::fs::read_to_string(root.join(relative)).expect(relative);
        for required in [
            "PREVIEW_MR",
            "PREVIEW_SHA",
            "PREVIEW_IMAGE",
            "PREVIEW_APP",
            "PREVIEW_HOST",
            "PREVIEW_BASE_NAMESPACE",
            "PREVIEW_CONTEXT",
            "PREVIEW_TTL_HOURS",
        ] {
            assert!(
                contents.contains(required),
                "{relative} missing required variable {required}"
            );
        }
        assert_command_order(
            relative,
            &contents,
            &[
                "preview discover-base",
                "preview render",
                "preview apply --dir",
                "--plan-only",
                "--dry-run",
                "kubectl",
                "rollout status",
                "preview router resolve",
                "preview comment",
                "preview cleanup plan",
                "preview cleanup apply",
            ],
        );
    }
}

fn assert_command_order(relative: &str, contents: &str, needles: &[&str]) {
    let mut offset = 0;
    for needle in needles {
        let Some(found) = contents[offset..].find(needle) else {
            panic!("{relative} missing command fragment {needle}");
        };
        offset += found + needle.len();
    }
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
