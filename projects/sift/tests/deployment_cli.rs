// HANDWRITE-BEGIN gap="sift-deployment-cli-tests" tracker="1606" reason="Verify all Dockerfile and layered Kubernetes artifact commands render expected contracts."
use std::process::Command;

fn sift(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args(args)
        .output()
        .expect("run sift deployment command");
    assert!(output.status.success(), "{args:?}: {output:?}");
    String::from_utf8(output.stdout).expect("utf-8 deployment output")
}

#[test]
fn layered_deployment_cli_renders_all_artifact_planes() {
    let dockerfile = sift(&["dockerfile", "render", "--variant", "source"]);
    assert!(dockerfile.contains("FROM rust:"));
    assert!(dockerfile.contains("next:"));

    let crd = sift(&["k8s", "crd", "render"]);
    assert!(crd.contains("kind: CustomResourceDefinition"));
    assert!(crd.contains("sifts.sift.axiom.dev"));

    let operator = sift(&["k8s", "operator", "render", "--namespace", "sift-system"]);
    assert!(operator.contains("kind: Deployment"));
    assert!(operator.contains("sift k8s operator run"));

    let instance = sift(&["k8s", "instance", "render", "--profile", "dev"]);
    assert!(instance.contains("kind: Sift"));
    assert!(instance.contains("replicasPerShard: 1"));
}

<!-- marker: sift-deployment-cli-tests path: projects/sift/tests/deployment_cli.rs reason: Verify all Dockerfile and layered Kubernetes artifact commands render expected contracts. -->
// HANDWRITE-END
