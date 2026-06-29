// SPEC-MANAGED: projects/lumen/tech-design/interfaces/cli/lumen-issue-search-view-create-shared-cli-standard.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:lumen-cli-convention" tracker="standardize-gap-projects-lumen-tests-cli-convention-rs" reason="CLI convention smoke test for the shared llm/upgrade/issue surface until the test generator owns binary-help assertions."
use std::process::Command;

#[test]
fn help_ships_standard_issue_group_not_report_issue() {
    let output = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .arg("--help")
        .output()
        .expect("run lumen --help");

    assert!(
        output.status.success(),
        "lumen --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let help = String::from_utf8_lossy(&output.stdout);
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
    let output = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(["issue", "--help"])
        .output()
        .expect("run lumen issue --help");

    assert!(
        output.status.success(),
        "lumen issue --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let help = String::from_utf8_lossy(&output.stdout);
    for command in ["search", "view", "create"] {
        assert!(help.contains(command), "missing `{command}` in:\n{help}");
    }
}
// HANDWRITE-END
