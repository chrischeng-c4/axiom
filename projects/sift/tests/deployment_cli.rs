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

// <HANDWRITE gap="missing-generator:unit-test" tracker="1675" reason="Verify collector rendering preserves the least-privilege node-log contract.">
#[test]
fn layered_deployment_cli_renders_all_artifact_planes() {
    let dockerfile = sift(&["dockerfile", "render", "--variant", "source"]);
    assert!(dockerfile.contains("FROM rust:"));
    assert!(dockerfile.contains("COPY --chown=65532:65532"));
    assert!(dockerfile.contains("next:"));

    let crd = sift(&["k8s", "crd", "render"]);
    assert!(crd.contains("kind: CustomResourceDefinition"));
    assert!(crd.contains("sifts.sift.axiom.dev"));

    let operator = sift(&["k8s", "operator", "render", "--namespace", "sift-system"]);
    assert!(operator.contains("kind: Deployment"));
    assert!(operator.contains("sift k8s operator run"));
    assert!(operator.contains("runAsNonRoot: true"));
    assert!(operator.contains("sift:0.1.0"));

    let instance = sift(&["k8s", "instance", "render", "--profile", "dev"]);
    assert!(instance.contains("kind: Sift"));
    assert!(instance.contains("replicasPerShard: 1"));
    assert!(instance.contains("sift:0.1.0"));

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
    assert!(collector.contains("path: /var/lib/sift-collector"));
    assert!(collector.contains("secretKeyRef:"));
    assert!(collector.contains("configMapKeyRef:"));
    assert!(collector.contains("fieldPath: spec.nodeName"));
    assert!(collector.contains("runAsNonRoot: true"));
    assert!(collector.contains("readOnlyRootFilesystem: true"));
    assert!(collector.contains("seccompProfile:"));
    assert!(collector.contains("drop: [\"ALL\"]"));
    assert!(collector.contains("requests: { cpu: 25m, memory: 64Mi }"));
    assert!(collector.contains("limits: { cpu: 500m, memory: 256Mi }"));
    assert!(!collector.contains("kind: ClusterRole"));
    assert!(!collector.contains("kind: Role"));
    assert!(!collector.contains("REPLACE_"));
}
// </HANDWRITE>

// HANDWRITE-END
