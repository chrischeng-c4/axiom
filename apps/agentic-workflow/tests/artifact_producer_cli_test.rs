// @spec apps/agentic-workflow/tech-design/surface/specs/aw-artifact-skeleton-fill-protocol.md#e2e-test
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:artifact-producer-cli" tracker="#1499" reason="The fixture drives the compiled CLI across three domain namespaces and isolated filesystem/git state."

use agentic_workflow::issues::LocalBackend;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn write_aw_config(root: &Path) {
    std::fs::write(
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

fn run_aw(root: &Path, args: &[&str], local_backend: bool) -> Value {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aw"));
    command.args(args).current_dir(root);
    if local_backend {
        command.env("AW_FIXTURE_LOCAL_BACKEND", "1");
    }
    let output = command.output().expect("run aw fixture command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "aw {} failed:\nstdout={stdout}\nstderr={stderr}",
        args.join(" ")
    );
    serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!(
            "aw {} did not emit one JSON value: {error}\nstdout={stdout}\nstderr={stderr}",
            args.join(" ")
        )
    })
}

fn assert_common_contract(root: &Path, value: &Value, producer: &str) {
    let artifact = &value["artifact"];
    assert_eq!(artifact["schema_version"], "aw.artifact-producer.v1");
    assert_eq!(artifact["identity"]["producer"], producer);
    assert_eq!(artifact["skeleton"]["initialized"], true);
    assert!(artifact["fill_slots"]
        .as_array()
        .is_some_and(|v| !v.is_empty()));
    assert!(artifact["validation"]["command"]
        .as_str()
        .is_some_and(|command| command.starts_with("aw ")));
    assert!(artifact["next"]["command"]
        .as_str()
        .is_some_and(|command| command.starts_with("aw ")));
    let skeleton = artifact["skeleton"]["path"].as_str().unwrap();
    assert!(artifact_path_exists(root, skeleton));
    for slot in artifact["fill_slots"].as_array().unwrap() {
        assert!(artifact_path_exists(
            root,
            slot["payload_path"].as_str().unwrap(),
        ));
    }
}

fn artifact_path_exists(root: &Path, path: &str) -> bool {
    let path = Path::new(path);
    if path.is_absolute() {
        path.exists()
    } else {
        root.join(path).exists()
    }
}

#[test]
fn wi_create_emits_cli_owned_skeleton_and_bounded_markdown_slot() {
    let temp = tempfile::tempdir().unwrap();
    write_aw_config(temp.path());
    let envelope = run_aw(
        temp.path(),
        &[
            "wi",
            "create",
            "--title",
            "Artifact producer WI fixture",
            "--type",
            "change",
            "--project",
            "demo",
            "--priority",
            "p1",
        ],
        true,
    );
    assert_common_contract(temp.path(), &envelope, "work_item");
    assert_eq!(
        envelope["artifact"]["fill_slots"][0]["format"],
        "markdown_fragment"
    );
    assert_eq!(envelope["artifact"]["fill_slots"][0]["id"], "all");
}

#[test]
fn ec_draft_emits_cli_owned_skeleton_and_structured_slots() {
    let temp = tempfile::tempdir().unwrap();
    write_aw_config(temp.path());
    let envelope = run_aw(
        temp.path(),
        &[
            "ec",
            "draft",
            "artifact-producer-ec-fixture",
            "--project",
            "demo",
            "--capability-id",
            "artifact-producer",
            "--claim-id",
            "ec-fixture",
            "--command",
            "cargo test -p demo",
            "--json",
        ],
        false,
    );
    assert_common_contract(temp.path(), &envelope, "external_contract");
    assert!(envelope["artifact"]["fill_slots"]
        .as_array()
        .unwrap()
        .iter()
        .all(|slot| slot["format"] == "json_schema"));
    assert_eq!(
        envelope["artifact"]["validation"]["command"],
        "aw ec review --project demo"
    );
}

fn init_git_fixture(root: &Path) {
    assert!(Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    for (key, value) in [
        ("user.email", "artifact-producer@example.invalid"),
        ("user.name", "Artifact Producer Fixture"),
        ("commit.gpgsign", "false"),
    ] {
        assert!(Command::new("git")
            .args(["config", key, value])
            .current_dir(root)
            .status()
            .unwrap()
            .success());
    }
    std::fs::write(root.join("README.md"), "# Demo\n").unwrap();
    assert!(Command::new("git")
        .args(["add", "README.md", "aw.toml"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["commit", "-m", "seed artifact producer fixture"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["switch", "-c", "app/fixture"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
}

#[test]
fn td_create_emits_cli_owned_skeleton_structured_slots_and_ownership() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write_aw_config(root);
    init_git_fixture(root);

    let slug = "artifact-producer-td-fixture";
    let issue_path = LocalBackend::from_project_root(root)
        .issues_dir()
        .join("open")
        .join(format!("{slug}.md"));
    std::fs::create_dir_all(issue_path.parent().unwrap()).unwrap();
    std::fs::write(
        issue_path,
        format!(
            "---\nslug: {slug}\ntitle: Artifact producer TD fixture\nstate: open\ntype: change\nlabels: [\"app:demo\"]\n---\n\n## Problem\n\nProve the TD producer contract.\n\n## Capability Alignment\n\nCapability: Artifact producer\nCapability Gap: TD fixture\nProgress Evidence: emitted contract\n\n## Requirements\n\n- R1: Emit the common contract.\n\n## Scope\n\n### In Scope\n- TD skeleton.\n\n### Out of Scope\n- Product code.\n\n## Acceptance Criteria\n\n- AC1: Contract is observable.\n\n## Reference Context\n\n### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| aw-artifact-skeleton-fill-protocol.md | primary |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| artifact-producer-fixture | create | tech-design/specs/{slug}.md |\n"
        ),
    )
    .unwrap();

    let spec_path = format!("tech-design/specs/{slug}.md");
    let envelope = run_aw(
        root,
        &[
            "td",
            "create",
            slug,
            "--project",
            "demo",
            "--spec-path",
            &spec_path,
        ],
        true,
    );
    assert_common_contract(root, &envelope, "tech_design");
    let ownership = envelope["artifact"]["ownership_outputs"]
        .as_array()
        .unwrap();
    assert_eq!(ownership[0]["marker"], "CODEGEN-BEGIN/END");
    assert_eq!(ownership[1]["marker"], "HANDWRITE-BEGIN/END");
    assert_eq!(
        ownership[1]["required_fields"],
        serde_json::json!(["gap", "tracker", "reason"])
    );
}

// HANDWRITE-END
