// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-tests-kind-lifecycle-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use std::fs;
use std::path::Path;
use std::process::Command;

use preview::{render_files, RenderInput};

fn input() -> RenderInput {
    RenderInput {
        mr: 123,
        sha: "abc123".to_string(),
        image: "registry.k8s.io/e2e-test-images/agnhost:2.45".to_string(),
        app: "checkout".to_string(),
        host: "uat.example.com".to_string(),
        owner: "payments-sre".to_string(),
        ttl_hours: 2,
        control_namespace: "preview-system".to_string(),
        workload_identity: "preview-runner".to_string(),
    }
}

#[test]
fn kind_server_side_dry_run_accepts_rendered_lifecycle_objects() {
    if std::env::var("PREVIEW_KIND_E2E").as_deref() != Ok("1") {
        eprintln!("skipping kind lifecycle EC; set PREVIEW_KIND_E2E=1 to run");
        return;
    }

    let context = output(Command::new("kubectl").args(["config", "current-context"]));
    assert!(
        context.starts_with("kind-")
            || std::env::var("PREVIEW_ALLOW_NON_KIND").as_deref() == Ok("1"),
        "refusing to run kind EC outside a kind context; context={context:?}"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    for file in render_files(&input()).expect("render") {
        if !file.path.starts_with("k8s/") && file.path != "router/route-binding.yaml" {
            continue;
        }
        let path = dir.path().join(&file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, file.contents).expect("write manifest");
    }

    kubectl_server_side_dry_run(&dir.path().join("k8s/namespace.yaml"));
    kubectl_server_side_dry_run(&dir.path().join("k8s/deployment.yaml"));
    kubectl_server_side_dry_run(&dir.path().join("k8s/service.yaml"));
    kubectl_server_side_dry_run(&dir.path().join("router/route-binding.yaml"));
}

fn kubectl_server_side_dry_run(path: &Path) {
    let status = Command::new("kubectl")
        .args(["apply", "--dry-run=server", "-f"])
        .arg(path)
        .status()
        .unwrap_or_else(|err| panic!("kubectl apply dry-run failed to start: {err}"));
    assert!(
        status.success(),
        "kubectl dry-run failed for {}",
        path.display()
    );
}

fn output(command: &mut Command) -> String {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("command failed to start: {err}"));
    assert!(
        output.status.success(),
        "command failed: status={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// </HANDWRITE>
