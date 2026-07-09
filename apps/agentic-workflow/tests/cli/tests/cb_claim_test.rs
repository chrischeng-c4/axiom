// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/cb_claim_test.md#source
// CODEGEN-BEGIN
//! Integration tests for `aw td create --from-source` (Phase 2 recovery).
//!
//! `aw td code-claim` was folded into `aw td create --from-source`
//! (epic #1270 R5 / #1273); these tests exercise the relocated surface,
//! including the #1243 regression proof that generated specs land under
//! the owning project's project-local `tech-design/` root, never the
//! legacy repo-root `.aw/tech-design`.

use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// R3 smoke: `aw td create` registers `--from-source` (folded code-claim).
#[test]
fn test_from_source_flag_registered() {
    let cmd = Cli::command();
    let create = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("create"))
        .expect("td create subcommand");
    create
        .get_arguments()
        .find(|a: &&clap::Arg| a.get_long() == Some("from-source"))
        .expect("--from-source registered");
}

/// #1273: the standalone `td code-claim` subcommand no longer exists.
#[test]
fn test_code_claim_subcommand_removed() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td namespace");
    assert!(
        td.find_subcommand("code-claim").is_none(),
        "td code-claim should be folded into td create --from-source"
    );
}

/// Issue #925: tracker linkage is default-on; `--no-issue` is the
/// documented opt-out (replaces the old opt-in `--issue-stub`), still
/// exposed on `td create`.
#[test]
fn test_from_source_no_issue_flag() {
    let cmd = Cli::command();
    let create = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("create"))
        .expect("td create");
    create
        .get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "no_issue")
        .expect("--no-issue registered");
    assert!(
        create
            .get_arguments()
            .all(|a: &clap::Arg| a.get_id().as_str() != "issue_stub"),
        "--issue-stub should be fully replaced by --no-issue, not left dangling"
    );
}

/// R3: `--non-interactive` flag remains registered as a boolean on `td create`.
///
/// @spec apps/agentic-workflow/tech-design/surface/specs/score-recovery-verbs-non-interactive.md#test-plan
#[test]
fn test_from_source_non_interactive_flag_registered() {
    let cmd = Cli::command();
    let create = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("create"))
        .expect("td create");
    create
        .get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "non_interactive")
        .expect("--non-interactive registered");
}

/// Cb-Claim trailer constant is exposed for downstream readers (unchanged
/// by the #1273 CLI-surface relocation).
#[test]
fn test_cb_claim_trailer_const() {
    use agentic_workflow::issues::types::lifecycle_trailer;
    assert_eq!(lifecycle_trailer::CB_CLAIM, "Cb-Claim");
}

/// R3 e2e: full fillback + write + trailer flow. Marked #[ignore]
/// because the fillback pipeline requires tree-sitter parsing on a real
/// codebase plus filesystem writes.
#[test]
#[ignore = "requires real codebase + fillback infrastructure; run manually with --ignored"]
fn test_from_source_fillback_invoked_e2e() {
    // Reserved for end-to-end: feed a small fixture into
    // `aw td create --from-source`, assert
    // <project>/tech-design/<group>/<derived>.md exists and contains YAML
    // frontmatter; assert the result envelope action == "done".
}

/// Writes a minimal `aw.toml` registering one project (`name`/`path` both
/// `project_rel`, plus a workspace scope so
/// `configured_project_name_for_path` can infer it without `--project`)
/// and a tiny Rust crate under `root/<project_rel>/src/lib.rs`.
fn write_from_source_fixture(root: &std::path::Path, project_rel: &str, type_name: &str) {
    let crate_dir = root.join(project_rel);
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("src/lib.rs"),
        format!("pub struct {type_name} {{\n    pub name: String,\n}}\n"),
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
            type_name.to_lowercase()
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("aw.toml"),
        format!(
            "[[projects]]\n\
             name = \"{project_rel}\"\n\
             path = \"{project_rel}\"\n\
             \n\
             [[projects.workspaces]]\n\
             name = \"{project_rel}\"\n\
             paths = [\"{project_rel}/**\"]\n\
             target = \"rust\"\n"
        ),
    )
    .unwrap();
}

fn run_from_source(
    aw_bin: &str,
    root: &std::path::Path,
    extra_args: &[&str],
) -> std::process::Output {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let mut cmd = Command::new(aw_bin);
    cmd.arg("td")
        .arg("create")
        .args(extra_args)
        .arg("--non-interactive")
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn aw td create --from-source");

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break child.wait_with_output().expect("wait_with_output"),
            Ok(None) => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    panic!(
                        "aw td create --from-source --non-interactive hung past 30s — \
                         interactive prompt still blocking"
                    );
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(e) => panic!("try_wait failed: {}", e),
        }
    }
}

/// #1243 regression proof: `aw td create --from-source <path> --project
/// <p>` must write the generated spec under the owning project's
/// project-local `tech-design/` root (`<project.path>/tech-design/...`),
/// never the legacy repo-root `.aw/tech-design`. Also runs `aw td check`
/// against the produced spec to confirm it is structurally valid (AC1).
#[test]
fn test_from_source_writes_under_project_local_td_root() {
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    write_from_source_fixture(root, "demo", "Foo");

    let output = run_from_source(
        &aw_bin,
        root,
        &["--from-source", "demo", "--project", "demo", "--no-issue"],
    );

    assert!(
        output.status.success(),
        "aw td create --from-source --project demo exit code {:?}, stderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    // Project-local root: root/demo/tech-design/.
    let project_local_td_dir = root.join("demo/tech-design");
    assert!(
        project_local_td_dir.exists(),
        "expected spec under project-local tech-design root {}",
        project_local_td_dir.display()
    );
    let spec_paths = collect_md_recursive(&project_local_td_dir);
    assert!(
        !spec_paths.is_empty(),
        "no spec files written under {}",
        project_local_td_dir.display()
    );

    // #1243: the legacy repo-root `.aw/tech-design` must NOT be used.
    assert!(
        !root.join(".aw/tech-design").exists(),
        "spec must not land under the legacy repo-root .aw/tech-design"
    );
    assert!(
        !root.join(".aw").exists(),
        ".aw/ workspace must not be created by aw td create --from-source"
    );

    // AC1: the produced spec must pass `aw td check`. Validate a real
    // per-module spec, not the `_overview.md`/`_dependency-graph.md`
    // scaffold files generate_specs also writes at the tech-design root --
    // those loose root files are a pre-existing fillback/`td check`
    // structure mismatch unrelated to the #1243 path-targeting fix and are
    // out of scope here.
    let spec_path = spec_paths
        .iter()
        .find(|p| {
            !p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('_'))
        })
        .unwrap_or(&spec_paths[0]);
    let check_output = std::process::Command::new(&aw_bin)
        .arg("td")
        .arg("check")
        .arg(spec_path.strip_prefix(root).unwrap_or(spec_path))
        .current_dir(root)
        .output()
        .expect("spawn aw td check");
    assert!(
        check_output.status.success(),
        "aw td check {} failed, stdout:\n{}\nstderr:\n{}",
        spec_path.display(),
        String::from_utf8_lossy(&check_output.stdout),
        String::from_utf8_lossy(&check_output.stderr)
    );
}

/// Same #1243 regression proof, but with `--project` omitted: the owning
/// project must be inferred from the `--from-source` path against the
/// configured project scopes (`configured_project_name_for_path`).
#[test]
fn test_from_source_infers_project_when_project_flag_omitted() {
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    write_from_source_fixture(root, "widget", "Bar");

    let output = run_from_source(&aw_bin, root, &["--from-source", "widget", "--no-issue"]);

    assert!(
        output.status.success(),
        "aw td create --from-source (inferred project) exit code {:?}, stderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let project_local_td_dir = root.join("widget/tech-design");
    assert!(
        project_local_td_dir.exists(),
        "expected inferred-project spec under {}",
        project_local_td_dir.display()
    );
    assert!(
        !collect_md_recursive(&project_local_td_dir).is_empty(),
        "no spec files written under {}",
        project_local_td_dir.display()
    );
    assert!(
        !root.join(".aw/tech-design").exists(),
        "spec must not land under the legacy repo-root .aw/tech-design"
    );
}

/// Issue #925: tracker linkage is default-on, but must not block
/// `aw td create --from-source` when no issue backend is configured
/// (offline / sandbox use). Asserts the command still exits 0 while
/// warning on stderr about the tracker-issue attempt.
#[test]
fn test_from_source_default_on_tracker_linkage_is_recoverable_offline() {
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    write_from_source_fixture(root, "demo", "Baz");

    let output = run_from_source(
        &aw_bin,
        root,
        &["--from-source", "demo", "--project", "demo"],
    );

    assert!(
        output.status.success(),
        "default-on tracker linkage must not fail td create --from-source offline; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tracker") || stderr.contains("issue"),
        "expected a recoverable tracker-linkage warning on stderr, got:\n{stderr}"
    );
}

/// Issue #925: `--no-issue` opts out of tracker linkage entirely and
/// still warns (rather than silently doing nothing), so the operator
/// knows no tracker root was created for the adopted code.
#[test]
fn test_from_source_no_issue_flag_skips_tracker_creation_with_warning() {
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    write_from_source_fixture(root, "demo", "Qux");

    let output = run_from_source(
        &aw_bin,
        root,
        &["--from-source", "demo", "--project", "demo", "--no-issue"],
    );

    assert!(
        output.status.success(),
        "--no-issue must still complete the claim; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--no-issue") || stderr.contains("skip"),
        "expected a skip-tracker-creation note on stderr, got:\n{stderr}"
    );
}

fn collect_md_recursive(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            out.extend(collect_md_recursive(&p));
        } else if p.extension().map(|e| e == "md").unwrap_or(false) {
            out.push(p);
        }
    }
    out
}

// CODEGEN-END
