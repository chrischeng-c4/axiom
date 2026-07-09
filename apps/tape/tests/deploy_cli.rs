// HANDWRITE-BEGIN gap="missing-generator:unit-test:31f1a60c" tracker="pending-tracker" reason="Offline deploy-CLI tests driving the COMPILED tape binary in the default build: every k8s/dockerfile render verb succeeds and round-trips serde_yaml; dockerfile source/release outputs equal the committed fixtures (+ --version substitution); the CRD render is structural-schema safe; operator run without the feature exits nonzero with the rebuild hint; the llm topic names the deploy verbs."
//! Offline deploy-CLI surface driven against the COMPILED `tape` binary in
//! the DEFAULT (kube-free) build (#1328): every `tape k8s ... render` and
//! `tape dockerfile render` verb succeeds offline and emits parseable
//! YAML/Dockerfiles; the committed Dockerfile fixtures are reproduced
//! byte-for-byte (relay #1208 pattern); the CRD render is structural-schema
//! safe; and the llm operations topic names the deploy verbs. No server, no
//! network, no live kind cluster (see tech-design for the scoped-down note).

use std::process::Command;

use serde::Deserialize;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tape"))
        .args(args)
        .output()
        .expect("run tape binary")
}

fn stdout(args: &[&str]) -> String {
    let out = run(args);
    assert!(
        out.status.success(),
        "`tape {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// Parse every YAML document in a (possibly multi-doc) stream, asserting the
/// stream is non-empty and each document is a mapping.
fn parse_yaml_docs(what: &str, yaml: &str) -> Vec<serde_yaml::Value> {
    let mut docs = Vec::new();
    for doc in serde_yaml::Deserializer::from_str(yaml) {
        let value = serde_yaml::Value::deserialize(doc)
            .unwrap_or_else(|e| panic!("{what}: YAML document does not parse: {e}"));
        if !value.is_null() {
            assert!(value.is_mapping(), "{what}: document is not a mapping");
            docs.push(value);
        }
    }
    assert!(!docs.is_empty(), "{what}: no YAML documents emitted");
    docs
}

/// R2: every render verb succeeds offline in the default build and
/// round-trips through serde_yaml; `operator run` without the feature exits
/// nonzero with the rebuild hint.
#[test]
fn render_verbs_emit_parseable_yaml_offline() {
    // CRD render: one CustomResourceDefinition document.
    let crd = parse_yaml_docs("crd render", &stdout(&["k8s", "crd", "render"]));
    assert_eq!(crd[0]["kind"], "CustomResourceDefinition");

    // Operator render: the control plane, namespace-substituted.
    let ops = parse_yaml_docs(
        "operator render",
        &stdout(&["k8s", "operator", "render", "--namespace", "tape-ops"]),
    );
    let kinds: Vec<&str> = ops.iter().filter_map(|d| d["kind"].as_str()).collect();
    for kind in [
        "Namespace",
        "ServiceAccount",
        "ClusterRole",
        "ClusterRoleBinding",
        "Deployment",
    ] {
        assert!(kinds.contains(&kind), "operator render missing {kind}");
    }
    let ns = ops.iter().find(|d| d["kind"] == "Namespace").unwrap();
    assert_eq!(ns["metadata"]["name"], "tape-ops", "namespace substituted");

    // Instance render: all four profiles emit a `kind: Tape` CR.
    for profile in ["dev", "staging", "prod", "template"] {
        let cr = parse_yaml_docs(
            &format!("instance render --profile {profile}"),
            &stdout(&["k8s", "instance", "render", "--profile", profile]),
        );
        assert_eq!(cr[0]["kind"], "Tape", "profile {profile}");
        assert_eq!(cr[0]["apiVersion"], "tape.dev/v1alpha1");
        assert!(cr[0]["spec"]["image"].is_string());
    }
    // The prod profile is the HA shape and dogfoods the auth wiring.
    let prod: serde_yaml::Value =
        serde_yaml::from_str(&stdout(&["k8s", "instance", "render", "--profile", "prod"])).unwrap();
    assert_eq!(prod["spec"]["replicasPerShard"], serde_yaml::Value::from(3));
    assert_eq!(prod["spec"]["voterCount"], serde_yaml::Value::from(3));
    assert_eq!(prod["spec"]["auth"], "required");

    // --name/--namespace/--image overrides flow into the CR.
    let custom: serde_yaml::Value = serde_yaml::from_str(&stdout(&[
        "k8s",
        "instance",
        "render",
        "--profile",
        "dev",
        "--name",
        "tape",
        "--namespace",
        "journals",
        "--image",
        "tape:pinned",
    ]))
    .unwrap();
    assert_eq!(custom["metadata"]["namespace"], "journals");
    assert_eq!(custom["spec"]["image"], "tape:pinned");

    // Dockerfile render succeeds for both variants (content asserted below).
    stdout(&["dockerfile", "render", "--variant", "source"]);
    stdout(&["dockerfile", "render", "--variant", "release"]);

    // operator run without the feature: nonzero exit + rebuild hint. (In an
    // `--features operator` build the verb runs the real controller — never
    // invoke it from a test, it would watch whatever cluster kubeconfig names.)
    #[cfg(not(feature = "operator"))]
    {
        let out = run(&["k8s", "operator", "run"]);
        assert!(
            !out.status.success(),
            "operator run must fail without --features operator"
        );
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("--features operator"),
            "rebuild hint names the feature"
        );
    }
}

/// R1: `--variant source` reproduces the committed `Dockerfile`
/// byte-for-byte, `--variant release` the committed `Dockerfile.release`, and
/// an explicit `--version` flows into the ARG + tag lines.
#[test]
fn dockerfile_render_reproduces_committed_fixtures() {
    let rendered = stdout(&["dockerfile", "render", "--variant", "source"]);
    assert_eq!(
        rendered,
        include_str!("../Dockerfile"),
        "tape dockerfile render --variant source == committed Dockerfile"
    );

    let rendered = stdout(&["dockerfile", "render", "--variant", "release"]);
    assert_eq!(
        rendered,
        include_str!("../Dockerfile.release"),
        "tape dockerfile render --variant release == committed Dockerfile.release"
    );

    let pinned = stdout(&[
        "dockerfile",
        "render",
        "--variant",
        "release",
        "--version",
        "9.9.9",
    ]);
    assert!(
        pinned.contains("ARG TAPE_VERSION=tape@9.9.9"),
        "pinned ARG: {pinned}"
    );
    assert!(
        pinned.contains("-t tape:9.9.9"),
        "pinned image tag: {pinned}"
    );
}

/// R3: the rendered CRD (the committed fixture in this default build) is
/// Kubernetes structural-schema safe and declares the Tape kind.
#[test]
fn crd_render_is_structural_schema_safe() {
    let yaml = stdout(&["k8s", "crd", "render"]);
    assert!(
        !yaml.contains("uint32"),
        "CRD must not carry format: uint32"
    );
    assert!(
        !yaml.contains("uint64"),
        "CRD must not carry format: uint64"
    );
    assert!(
        yaml.contains("minimum"),
        "normalized uints keep a minimum floor"
    );

    let doc: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("CRD parses as YAML");
    assert_eq!(doc["kind"], "CustomResourceDefinition");
    assert_eq!(doc["spec"]["group"], "tape.dev");
    assert_eq!(doc["spec"]["names"]["kind"], "Tape");
    assert_eq!(doc["metadata"]["name"], "tapes.tape.dev");
}

/// R10: the llm operations topic documents the deploy verbs so the outline
/// stays honest about the CLI surface.
#[test]
fn llm_topic_names_deploy_verbs() {
    let topic = stdout(&["llm", "--topic", "operations"]);
    for needle in [
        "k8s crd render",
        "k8s operator render",
        "k8s operator run",
        "k8s instance render",
        "dockerfile render",
    ] {
        assert!(
            topic.contains(needle),
            "operations topic missing `{needle}`"
        );
    }
}
// HANDWRITE-END
