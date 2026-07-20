// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-cli-contract" tracker="#766" reason="CLI convention, agent onboarding, offline OpenAPI, and shared client-codegen regression proof."
use std::process::Command;

fn defer() -> Command {
    Command::new(env!("CARGO_BIN_EXE_defer"))
}

#[test]
fn help_exposes_standard_and_domain_surfaces() {
    let output = defer().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for command in [
        "serve",
        "spec",
        "llm",
        "upgrade",
        "issue",
        "queue",
        "task",
        "dispatch",
        "backup",
        "k8s",
        "dockerfile",
    ] {
        assert!(stdout.contains(command), "missing {command} in --help");
    }
}

#[test]
fn serve_exposes_shared_structured_log_configuration() {
    let output = defer().args(["serve", "--help"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--log-format"));
    assert!(stdout.contains("pretty"));
    assert!(stdout.contains("json"));
}

#[test]
fn deploy_artifacts_render_by_lifecycle_layer() {
    let crd = defer().args(["k8s", "crd", "render"]).output().unwrap();
    assert!(crd.status.success());
    assert!(String::from_utf8(crd.stdout)
        .unwrap()
        .contains("kind: CustomResourceDefinition"));

    let operator = defer()
        .args(["k8s", "operator", "render", "--namespace", "control"])
        .output()
        .unwrap();
    let operator = String::from_utf8(operator.stdout).unwrap();
    assert!(operator.contains("kind: Deployment"));
    assert!(operator.contains("namespace: control"));
    assert!(operator.contains("name: POD_NAME"));
    assert!(operator.contains("name: POD_NAMESPACE"));
    assert!(operator.contains("fieldPath: metadata.namespace"));

    let instance = defer()
        .args(["k8s", "instance", "render", "--profile", "prod"])
        .output()
        .unwrap();
    let instance = String::from_utf8(instance.stdout).unwrap();
    assert!(instance.contains("kind: Defer"));
    assert!(instance.contains("replicasPerShard: 3"));
    assert!(instance.contains("backup:"));

    for variant in ["source", "release"] {
        let dockerfile = defer()
            .args(["dockerfile", "render", "--variant", variant])
            .output()
            .unwrap();
        assert!(dockerfile.status.success());
        assert!(String::from_utf8(dockerfile.stdout)
            .unwrap()
            .contains("ENTRYPOINT"));
    }

    let release = defer()
        .args(["dockerfile", "render", "--variant", "release"])
        .output()
        .expect("render release Dockerfile");
    assert!(release.status.success());
    assert!(String::from_utf8(release.stdout)
        .expect("release Dockerfile stdout")
        .contains(&format!(
            "ARG DEFER_VERSION=defer@{}",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn llm_outline_advertises_cross_scope_topics_and_terminates() {
    let output = defer()
        .args(["llm", "--topic", "outline"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for topic in ["workflow", "api", "delivery", "ha", "auth"] {
        assert!(stdout.contains(&format!("`{topic}`")), "missing {topic}");
    }
    assert!(stdout.contains("next: done"));
}

#[test]
fn offline_spec_and_typed_client_generation_use_one_contract() {
    let output = defer()
        .args(["spec", "--format", "openapi"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("/v1/queues/{queue}/tasks"));
    assert!(stdout.contains("next: done"));

    let out = tempfile::tempdir().unwrap();
    let generated = defer()
        .args([
            "spec",
            "gen",
            "--lang",
            "ts",
            "--out",
            out.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    assert!(out.path().read_dir().unwrap().next().is_some());
    assert!(String::from_utf8(generated.stdout)
        .unwrap()
        .contains("next: done"));
}

// HANDWRITE-END
