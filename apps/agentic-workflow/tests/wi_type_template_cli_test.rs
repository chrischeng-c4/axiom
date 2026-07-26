// @spec apps/agentic-workflow/tech-design/src/agentic_workflow/work_items/templates.py
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:wi-type-template" tracker="#2594" reason="The compiled CLI fixture proves hard Spike and Report profiles and Report intake's Capability Alignment exemption."

use agentic_workflow::issues::{IssueBackend, LocalBackend};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn write_project(root: &Path) {
    fs::write(
        root.join("aw.toml"),
        r#"
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
label = "app:demo"
path = "."
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "demo"
paths = ["**"]
target = "rust"
"#,
    )
    .unwrap();
}

fn run_aw(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(args)
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .env("AW_DISABLE_CAP", "1")
        .output()
        .expect("run repo-built aw")
}

fn successful_json(output: &Output, command: &str) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "{command} failed:\nstdout={stdout}\nstderr={stderr}"
    );
    serde_json::from_str(stdout.trim()).unwrap()
}

#[tokio::test]
async fn spike_and_report_use_hard_type_specific_profiles() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    let backend = LocalBackend::from_project_root(root.path());

    let spike = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "create",
                "--title",
                "Which retry policy should the runner use?",
                "--type",
                "spike",
                "--project",
                "demo",
            ],
        ),
        "create spike",
    );
    let spike = backend
        .get(spike["slug"].as_str().unwrap())
        .await
        .unwrap()
        .unwrap();
    for heading in [
        "## Question",
        "## Evidence Plan",
        "## Exit Criteria",
        "## Timebox",
    ] {
        assert!(spike.body.contains(heading), "missing {heading}");
    }
    assert!(!spike.body.contains("## Capability Alignment"));

    let report = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "create",
                "--title",
                "CLI exits without a remediation command",
                "--type",
                "report",
                "--project",
                "demo",
            ],
        ),
        "create report",
    );
    let report = backend
        .get(report["slug"].as_str().unwrap())
        .await
        .unwrap()
        .unwrap();
    for heading in ["## Repro", "## Diagnostics", "## Expected vs Actual"] {
        assert!(report.body.contains(heading), "missing {heading}");
    }
    assert!(!report.body.contains("## Capability Alignment"));

    let invalid = run_aw(
        root.path(),
        &[
            "wi",
            "create",
            "--title",
            "Cross-profile report",
            "--type",
            "report",
            "--project",
            "demo",
            "--body",
            "## Repro\n\n- reproduce\n\n## Diagnostics\n\n- logs\n\n## Expected vs Actual\n\nExpected: success\nActual: failure\n\n## Scope\n\n- forbidden\n",
        ],
    );
    assert!(!invalid.status.success());
    assert!(
        String::from_utf8_lossy(&invalid.stderr)
            .contains("report work-item accepts only these H2 sections"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&invalid.stderr)
    );
}

// HANDWRITE-END
