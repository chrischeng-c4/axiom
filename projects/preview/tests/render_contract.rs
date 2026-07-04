// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-tests-render-contract-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use preview::render::{cleanup_plan, preview_environment};
use preview::{render_files, CleanupAction, RenderInput};

fn input() -> RenderInput {
    RenderInput {
        mr: 123,
        sha: "abc123".to_string(),
        image: "us-docker.pkg.dev/acme/uat/app:abc123".to_string(),
        app: "checkout".to_string(),
        host: "uat.example.com".to_string(),
        base_namespace: "uat-base".to_string(),
        owner: "payments-sre".to_string(),
        ttl_hours: 48,
        control_namespace: "preview-system".to_string(),
        workload_identity: "preview-runner".to_string(),
        base_contract: None,
    }
}

#[test]
fn render_creates_gke_contract_files() {
    let files = render_files(&input()).expect("render files");
    let paths: Vec<_> = files.iter().map(|file| file.path.as_str()).collect();

    assert!(paths.contains(&"spec/preview-environment.yaml"));
    assert!(paths.contains(&"plans/workload-clone.json"));
    assert!(paths.contains(&"plans/manifest-inventory.json"));
    assert!(paths.contains(&"k8s/namespace.yaml"));
    assert!(paths.contains(&"k8s/service-account.yaml"));
    assert!(paths.contains(&"k8s/resource-quota.yaml"));
    assert!(paths.contains(&"k8s/limit-range.yaml"));
    assert!(paths.contains(&"k8s/workload-role.yaml"));
    assert!(paths.contains(&"k8s/workload-role-binding.yaml"));
    assert!(paths.contains(&"k8s/deployment.yaml"));
    assert!(paths.contains(&"k8s/service.yaml"));
    assert!(paths.contains(&"router/route-binding.yaml"));
    assert!(paths.contains(&"mr-comment.md"));
    assert!(paths.contains(&"cleanup-plan.json"));

    let namespace = files
        .iter()
        .find(|file| file.path == "k8s/namespace.yaml")
        .expect("namespace");
    assert!(namespace.contents.contains("name: uat-mr-123"));
    assert!(namespace.contents.contains("preview.cclab.dev/mr: '123'"));
    assert!(namespace
        .contents
        .contains("preview.cclab.dev/base-namespace: uat-base"));

    let inventory = files
        .iter()
        .find(|file| file.path == "plans/manifest-inventory.json")
        .expect("manifest inventory");
    let inventory: serde_json::Value =
        serde_json::from_str(&inventory.contents).expect("manifest inventory json");
    assert_eq!(inventory["namespace"], "uat-mr-123");
    assert_eq!(inventory["entries"][0]["path"], "k8s/namespace.yaml");
    assert_eq!(inventory["entries"][8]["path"], "router/route-binding.yaml");
}

#[test]
fn clone_plan_references_base_workload_without_cloning_runtime_identity() {
    let files = render_files(&input()).expect("render files");
    let clone_plan = files
        .iter()
        .find(|file| file.path == "plans/workload-clone.json")
        .expect("clone plan");
    let clone_plan: serde_json::Value =
        serde_json::from_str(&clone_plan.contents).expect("clone plan json");

    assert_eq!(clone_plan["source"]["namespace"], "uat-base");
    assert_eq!(clone_plan["source"]["workload"], "checkout");
    assert_eq!(clone_plan["target"]["namespace"], "uat-mr-123");
    assert_eq!(clone_plan["target"]["routeTarget"], "mr-123");
    assert!(clone_plan["copyPolicy"]["exclude"]
        .as_array()
        .expect("exclude")
        .iter()
        .any(|value| value == "clusterIP"));
}

#[test]
fn route_binding_uses_target_not_namespace_cookie() {
    let files = render_files(&input()).expect("render files");
    let binding = files
        .iter()
        .find(|file| file.path == "router/route-binding.yaml")
        .expect("route binding");

    assert!(binding.contents.contains("target: mr-123"));
    assert!(binding.contents.contains("namespace: uat-mr-123"));
    assert!(binding.contents.contains("cookie: uat_target"));
    assert!(binding.contents.contains("header: X-UAT-Target"));
}

#[test]
fn cleanup_plan_marks_closed_mr_for_namespace_delete() {
    let env = preview_environment(&input());
    let plan = cleanup_plan(&env, true);

    assert_eq!(plan.action, CleanupAction::Delete);
    assert!(plan.delete_namespace);
    assert!(plan.delete_route_binding);
    assert_eq!(plan.namespace, "uat-mr-123");
    assert_eq!(plan.route_target, "mr-123");
    assert_eq!(
        plan.protected_namespaces,
        vec!["uat-base".to_string(), "preview-system".to_string()]
    );
}

#[test]
fn render_is_deterministic_for_same_mr_and_isolated_for_different_mrs() {
    let first = render_files(&input()).expect("first render");
    let second = render_files(&input()).expect("second render");
    assert_eq!(first, second);

    let mut other_input = input();
    other_input.mr = 456;
    other_input.sha = "def456".to_string();
    let other = render_files(&other_input).expect("other render");

    let first_namespace = first
        .iter()
        .find(|file| file.path == "k8s/namespace.yaml")
        .expect("first namespace");
    let other_namespace = other
        .iter()
        .find(|file| file.path == "k8s/namespace.yaml")
        .expect("other namespace");
    let other_binding = other
        .iter()
        .find(|file| file.path == "router/route-binding.yaml")
        .expect("other route binding");

    assert!(first_namespace.contents.contains("name: uat-mr-123"));
    assert!(other_namespace.contents.contains("name: uat-mr-456"));
    assert!(other_binding.contents.contains("target: mr-456"));
    assert!(!other_binding.contents.contains("target: mr-123"));
}

// </HANDWRITE>
