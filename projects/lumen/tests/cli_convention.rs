// SPEC-MANAGED: projects/lumen/tech-design/interfaces/cli/lumen-issue-search-view-create-shared-cli-standard.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:lumen-cli-convention" tracker="standardize-gap-projects-lumen-tests-cli-convention-rs" reason="CLI convention smoke test for the shared llm/upgrade/issue surface until the test generator owns binary-help assertions."
use std::process::Command;

fn run_lumen(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("run lumen {args:?}: {err}"));

    assert!(
        output.status.success(),
        "lumen {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("lumen stdout is utf8")
}

#[test]
fn help_ships_standard_issue_group_not_report_issue() {
    let help = run_lumen(&["--help"]);
    for command in ["llm", "upgrade", "issue"] {
        assert!(help.contains(command), "missing `{command}` in:\n{help}");
    }
    assert!(
        !help.contains("report-issue"),
        "deprecated report-issue command still appears in:\n{help}"
    );
}

#[test]
fn issue_help_lists_search_view_create() {
    let help = run_lumen(&["issue", "--help"]);
    for command in ["search", "view", "create"] {
        assert!(help.contains(command), "missing `{command}` in:\n{help}");
    }
}

#[test]
fn help_ships_dockerfile_and_layered_k8s_groups() {
    let help = run_lumen(&["--help"]);
    for command in ["dockerfile", "k8s"] {
        assert!(help.contains(command), "missing `{command}` in:\n{help}");
    }

    let k8s_help = run_lumen(&["k8s", "--help"]);
    for layer in ["crd", "operator", "instance"] {
        assert!(
            k8s_help.contains(layer),
            "missing `{layer}` in:\n{k8s_help}"
        );
    }

    let operator_help = run_lumen(&["k8s", "operator", "--help"]);
    for command in ["run", "render"] {
        assert!(
            operator_help.contains(command),
            "missing `{command}` in:\n{operator_help}"
        );
    }
}

#[test]
fn dockerfile_render_release_sets_version_and_strips_markers() {
    let rendered = run_lumen(&[
        "dockerfile",
        "render",
        "--variant",
        "release",
        "--version",
        "9.9.9",
    ]);

    assert!(rendered.contains("ARG LUMEN_VERSION=lumen@9.9.9"));
    assert!(rendered.contains("-t lumen:9.9.9"));
    assert!(!rendered.contains("SPEC-MANAGED"));
    assert!(!rendered.contains("CODEGEN-BEGIN"));
    assert!(!rendered.contains("CODEGEN-END"));
}

#[test]
fn k8s_crd_render_is_offline() {
    let rendered = run_lumen(&["k8s", "crd", "render"]);

    assert!(rendered.contains("kind: CustomResourceDefinition"));
    assert!(rendered.contains("name: lumens.lumen.dev"));
}

#[test]
fn k8s_instance_render_prod_accepts_app_namespace_overrides() {
    let rendered = run_lumen(&[
        "k8s",
        "instance",
        "render",
        "--profile",
        "prod",
        "--namespace",
        "search-prod",
        "--name",
        "catalog",
        "--image",
        "registry.example/lumen:9.9.9",
    ]);

    for expected in [
        "kind: Lumen",
        "  name: catalog",
        "  namespace: search-prod",
        "  image: registry.example/lumen:9.9.9",
        "  auth: required",
        "  observability: true",
    ] {
        assert!(
            rendered.contains(expected),
            "missing `{expected}` in:\n{rendered}"
        );
    }
}
// HANDWRITE-END
