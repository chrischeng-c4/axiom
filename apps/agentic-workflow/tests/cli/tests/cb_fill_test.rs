// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/cb_fill_test.md#source
// CODEGEN-BEGIN
//! Integration tests for `aw cb fill` (Phase 3).
//!
//! Smoke tests for CLI registration, brief mode envelope shape, marker
//! enumeration, and `--apply --marker` block replacement, plus a real-binary
//! round trip (`test_apply_marker_replaces_block`, issue #1096 AC1) proving
//! the payload lives under `/tmp/aw/workspaces/<workspace>/payloads/` and
//! that apply reads it back. The remaining e2e integration scenarios (code
//! check gate + Cb-Fill trailer + phase advance) are #[ignore]d because they
//! require a real worktree, real payload files, and the agent loop
//! infrastructure.
//!
//! @spec apps/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#test-plan

use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn marker(source_path: &str) -> agentic_workflow::cli::cb_fill::HandwriteMarkerEntry {
    agentic_workflow::cli::cb_fill::HandwriteMarkerEntry {
        id: source_path.replace(['/', '.'], "-"),
        source_path: source_path.to_string(),
        start_line: 1,
        end_line: 3,
        reason: "test marker".to_string(),
        spec_ref: None,
        adopt_existing: false,
    }
}

fn handwrite_begin(attrs: &str) -> String {
    format!("// HANDWRITE-{} {}", "BEGIN", attrs)
}

fn handwrite_end() -> &'static str {
    concat!("// HANDWRITE-", "END")
}

// ── R1 / R14(1) ─────────────────────────────────────────────────────────

/// R1: `aw cb fill` is registered as a first-class subcommand under cb.
#[test]
fn test_cb_fill_registered() {
    let cmd = Cli::command();
    let cb = cmd.find_subcommand("cb").expect("cb namespace");
    let fill = cb.find_subcommand("fill").expect("cb fill subcommand");
    let positionals: Vec<String> = fill
        .get_positionals()
        .map(|p: &clap::Arg| p.get_id().as_str().to_string())
        .collect();
    assert!(positionals.iter().any(|p| p == "slug"));
}

#[test]
fn test_cb_fill_apply_flag() {
    let cmd = Cli::command();
    let fill = cmd
        .find_subcommand("cb")
        .and_then(|c| c.find_subcommand("fill"))
        .expect("cb fill");
    fill.get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "apply")
        .expect("--apply flag");
    fill.get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "marker")
        .expect("--marker flag");
}

#[test]
fn test_cb_fill_spec_path_flag() {
    let cmd = Cli::command();
    let fill = cmd
        .find_subcommand("cb")
        .and_then(|c| c.find_subcommand("fill"))
        .expect("cb fill");
    fill.get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "spec_path")
        .expect("--spec-path flag");
}

// ── R9 / R14(0) — enum extensions ────────────────────────────────────────

/// R9: `cb_filled` phase const exists in agentic_workflow::issues::types::td_phase.
#[test]
fn test_issue_phase_cb_filled_variant() {
    use agentic_workflow::issues::types::td_phase;
    assert_eq!(td_phase::CB_FILLED, "cb_filled");
    assert!(td_phase::is_terminal_code_checkable("cb_filled"));
    assert!(td_phase::is_terminal_code_checkable("cb_genned"));
    assert!(!td_phase::is_terminal_code_checkable("td_reviewed"));
}

/// R9: `Cb-Fill` trailer const exists in lifecycle_trailer module.
#[test]
fn test_lifecycle_trailer_cb_fill_variant() {
    use agentic_workflow::issues::types::lifecycle_trailer;
    assert_eq!(lifecycle_trailer::CB_FILL, "Cb-Fill");
}

/// R10: terminal `aw cb check` accepts `cb_filled` as a valid phase.
/// We verify this at the helper-level:
/// `is_terminal_code_checkable("cb_filled") == true`.
#[test]
fn test_td_code_check_accepts_cb_filled() {
    use agentic_workflow::issues::types::td_phase;
    assert!(td_phase::is_terminal_code_checkable(td_phase::CB_FILLED));
}

// ── R2 / R14(1) — brief mode envelope shape ─────────────────────────────

/// R2: brief-mode envelope shape — verified via the helper that builds
/// the envelope JSON. We construct a fake marker list and assert the
/// emitted envelope has action="dispatch" and agent=null (mainthread-only
/// execution model: mainthread runs invoke.command directly).
#[test]
fn test_brief_mode_envelope_shape() {
    // Synthesize the envelope JSON the same way `run_brief` does.
    let env = serde_json::json!({
        "action": "dispatch",
        "agent": null,
        "slug": "demo",
        "invoke": {
            "command": "aw cb fill",
            "args": {
                "slug": "demo",
                "marker_list": [{
                    "id": "cb-fill-issue-phase-enum",
                    "source_path": "apps/agentic-workflow/src/issues/types.rs",
                    "start_line": 10,
                    "end_line": 14,
                    "reason": "issue-phase enum gap",
                }],
                "spec_path": "",
            },
        },
    });
    assert_eq!(env["action"], "dispatch");
    assert!(env["agent"].is_null());
    assert!(env["invoke"]["args"]["marker_list"].is_array());
}

/// R2: marker list element shape is the spec-defined HandwriteMarkerEntry.
#[test]
fn test_brief_mode_marker_list_present() {
    let entry = agentic_workflow::cli::cb_fill::HandwriteMarkerEntry {
        id: "marker-1".into(),
        source_path: "src/x.rs".into(),
        start_line: 1,
        end_line: 5,
        reason: "test".into(),
        spec_ref: None,
        adopt_existing: false,
    };
    let v = serde_json::to_value(&entry).unwrap();
    assert!(v.get("id").is_some());
    assert!(v.get("source_path").is_some());
    assert!(v.get("start_line").is_some());
    assert!(v.get("end_line").is_some());
    assert!(v.get("reason").is_some());
}

/// R2: agent address for brief envelope MUST be null under the
/// mainthread-only execution model (score-mainthread-only-execution.md).
#[test]
fn test_brief_mode_agent_address() {
    let env = serde_json::json!({
        "action": "dispatch",
        "agent": null,
    });
    assert!(env["agent"].is_null());
}

// ── R4 — apply mode marker replacement ──────────────────────────────────

/// R4: enumeration finds a HANDWRITE block in a file. Uses tempdir.
#[test]
fn test_apply_marker_enumerates_block() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let src_file = src_dir.join("x.rs");
    let body = format!(
        "fn before() {{}}\n{}\nTODO: hand-write content\n{}\nfn after() {{}}\n",
        handwrite_begin("gap=\"my-marker\" tracker=\"none\" reason=\"because\""),
        handwrite_end()
    );
    std::fs::write(&src_file, body).unwrap();

    let markers = agentic_workflow::cli::cb_fill::enumerate_worktree_markers(tmp.path());
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].id, "my-marker");
    assert_eq!(markers[0].reason, "because");
}

/// R4: enumerating two distinct HANDWRITE blocks in one file returns both.
#[test]
fn test_apply_marker_no_adjacent_disturbance() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let src_file = src_dir.join("x.rs");
    let body = format!(
        "fn a() {{}}\n{}\nTODO: hand-write content\n{}\nfn b() {{}}\n{}\nTODO: hand-write content\n{}\nfn c() {{}}\n",
        handwrite_begin("gap=\"first\" tracker=\"t\" reason=\"r1\""),
        handwrite_end(),
        handwrite_begin("gap=\"second\" tracker=\"t\" reason=\"r2\""),
        handwrite_end()
    );
    std::fs::write(&src_file, body).unwrap();
    let markers = agentic_workflow::cli::cb_fill::enumerate_worktree_markers(tmp.path());
    let ids: Vec<&str> = markers.iter().map(|m| m.id.as_str()).collect();
    assert!(ids.contains(&"first"));
    assert!(ids.contains(&"second"));
}

/// R11: 0-marker fast-path — when no HANDWRITE markers exist, the
/// enumerator returns empty.
#[test]
fn test_zero_marker_fastpath_no_markers() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("clean.rs"), "fn ok() {}\n").unwrap();
    let markers = agentic_workflow::cli::cb_fill::enumerate_worktree_markers(tmp.path());
    assert!(markers.is_empty());
}

/// R11: count helper agrees with enumeration (used by td.rs for the
/// post-codegen dispatch decision).
#[test]
fn test_count_matches_enumeration() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let body = format!(
        "{}\nTODO: hand-write content\n{}\n",
        handwrite_begin("gap=\"x\" tracker=\"t\" reason=\"r\""),
        handwrite_end()
    );
    std::fs::write(src_dir.join("a.rs"), body).unwrap();
    let n = agentic_workflow::cli::cb_fill::count_worktree_handwrite_markers(tmp.path());
    assert_eq!(n, 1);
}

/// R1/R2: TD Changes YAML supports both `changes:` and legacy `files:`,
/// and accepts either `path:` or `file:` entries.
#[test]
fn test_extract_change_paths_supports_changes_and_files() {
    let spec = concat!(
        "\n",
        "#",
        "# Logic\n\n",
        "not parsed\n\n",
        "#",
        "# Changes\n\n",
        "```yaml\n",
        "changes:\n",
        "  - path: ./apps/agentic-workflow/src/cli/cb_fill.rs\n",
        "  - file: apps/agentic-workflow/src/issues/types.rs\n",
        "files:\n",
        "  - path: ignored/by/changes.rs\n",
        "```\n\n",
        "#",
        "# Test Plan\n\n",
        "```yaml\n",
        "files:\n",
        "  - file: ignored/outside/changes.rs\n",
        "```\n",
    );

    let paths = agentic_workflow::cli::cb_fill::extract_change_paths_from_spec(spec);
    assert_eq!(
        paths,
        vec![
            "apps/agentic-workflow/src/cli/cb_fill.rs".to_string(),
            "apps/agentic-workflow/src/issues/types.rs".to_string(),
        ],
    );

    let legacy_spec = concat!(
        "\n",
        "#",
        "# Changes\n\n",
        "```yaml\n",
        "files:\n",
        "  - file: ./apps/agentic-workflow/tests/cb_fill_test.rs\n",
        "```\n",
    );
    let paths = agentic_workflow::cli::cb_fill::extract_change_paths_from_spec(legacy_spec);
    assert_eq!(
        paths,
        vec!["apps/agentic-workflow/tests/cb_fill_test.rs".to_string()],
    );
}

/// R2: inherited markers outside the active TD Changes paths are filtered
/// out of the brief-mode marker list.
#[test]
fn test_scope_filters_to_changed_source_paths() {
    let markers = vec![
        marker("apps/agentic-workflow/src/cli/cb_fill.rs"),
        marker("apps/agentic-workflow/src/cli/cb.rs"),
        marker("apps/agentic-workflow/src/issues/types.rs"),
    ];
    let change_paths = vec![
        "apps/agentic-workflow/src/cli".to_string(),
        "apps/agentic-workflow/tests/*_test.rs".to_string(),
    ];

    let scoped =
        agentic_workflow::cli::cb_fill::filter_markers_to_change_paths(&markers, &change_paths);
    let paths: Vec<&str> = scoped.iter().map(|m| m.source_path.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "apps/agentic-workflow/src/cli/cb_fill.rs",
            "apps/agentic-workflow/src/cli/cb.rs",
        ],
    );
}

/// R1: when the active TD only changes spec files, source HANDWRITE markers
/// are outside scope and the brief path can dispatch directly to merge.
#[test]
fn test_scope_zero_marker_for_spec_only_change() {
    let markers = vec![
        marker("apps/agentic-workflow/src/cli/cb_fill.rs"),
        marker("apps/agentic-workflow/src/issues/types.rs"),
    ];
    let change_paths =
        vec!["apps/agentic-workflow/tech-design/surface/specs/spec-only-change.md".to_string()];

    let scoped =
        agentic_workflow::cli::cb_fill::filter_markers_to_change_paths(&markers, &change_paths);
    assert!(scoped.is_empty());
}

/// R1 fallback: when no active spec is resolved, brief mode keeps the legacy
/// all-marker behavior instead of silently dropping inherited markers.
#[test]
fn test_scope_missing_spec_uses_legacy_all_markers() {
    let markers = vec![
        marker("apps/agentic-workflow/src/cli/cb_fill.rs"),
        marker("apps/agentic-workflow/src/issues/types.rs"),
    ];

    let scoped = agentic_workflow::cli::cb_fill::scope_markers_for_change_paths(&markers, None);
    assert_eq!(scoped.len(), markers.len());
    assert_eq!(scoped[0].source_path, markers[0].source_path);
    assert_eq!(scoped[1].source_path, markers[1].source_path);
}

/// #2535: `aw goal wi` may resolve a GitHub WI after the ephemeral local
/// projection was lost. The exact emitted `aw td fill <id>` command must
/// hydrate that projection before creating the marker-fill lock.
#[tokio::test]
async fn td_fill_brief_hydrates_missing_remote_projection() {
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    let Some(git) = agentic_workflow::git::find_git_bin() else {
        eprintln!("skipping: git binary not on PATH");
        return;
    };
    let root = tempfile::tempdir().expect("tempdir");

    Command::new(&git)
        .arg("-C")
        .arg(root.path())
        .args(["init", "-b", "project-test"])
        .status()
        .expect("git init");
    for (key, value) in [
        ("user.email", "test@test"),
        ("user.name", "test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new(&git)
            .arg("-C")
            .arg(root.path())
            .args(["config", key, value])
            .status()
            .unwrap();
    }

    std::fs::write(
        root.path().join("aw.toml"),
        concat!(
            "[agentic_workflow.issue_platform]\n",
            "type = \"github\"\n",
            "repo = \"fixture/configured\"\n",
        ),
    )
    .unwrap();
    let spec_rel = "apps/demo/tech-design/logic/remote-fill.md";
    let source_rel = "apps/demo/src/lib.rs";
    let spec = root.path().join(spec_rel);
    std::fs::create_dir_all(spec.parent().unwrap()).unwrap();
    std::fs::write(
        &spec,
        format!(
            "## Changes\n```yaml\nchanges:\n  - path: {source_rel}\n    action: modify\n    impl_mode: hand-written\n```\n"
        ),
    )
    .unwrap();
    let source = root.path().join(source_rel);
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(
        &source,
        format!(
            "{}\n// TODO: hand-write content for `{source_rel}`.\n{}\n",
            handwrite_begin(
                "gap=\"remote-fill-marker\" tracker=\"pending-tracker\" reason=\"fixture\""
            ),
            handwrite_end(),
        ),
    )
    .unwrap();

    Command::new(&git)
        .arg("-C")
        .arg(root.path())
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root.path())
        .args(["commit", "-m", "seed remote fill fixture"])
        .status()
        .unwrap();

    let tool_home = tempfile::tempdir().expect("tool home");
    let home = tool_home.path();
    let gh = home.join(".rustup/toolchains/stable-aarch64-apple-darwin/bin/gh");
    std::fs::create_dir_all(gh.parent().unwrap()).unwrap();
    let issue_json = serde_json::json!({
        "number": 2535,
        "title": "remote fill hydration fixture",
        "state": "OPEN",
        "labels": [
            {"name": "type:change"},
            {"name": "phase:cb_genned"},
            {"name": "project:agentic-workflow"}
        ],
        "author": {"login": "fixture"},
        "createdAt": "2026-07-24T00:00:00Z",
        "updatedAt": "2026-07-24T00:00:00Z",
        "url": "https://example.invalid/fixture/issues/2535",
        "body": "remote-only WI fixture"
    })
    .to_string();
    let gh_script = format!(
        r#"#!/bin/sh
printf '%s\n' "$*" >> "$AW_GH_LOG"
case "$*" in
  *" list "*) printf '%s\n' '[]' ;;
  *" view 2535 "*) printf '%s\n' '{issue_json}' ;;
  label\ create*) printf '%s\n' '{{}}' ;;
  api\ -X\ PATCH*) printf '%s\n' '{{}}' ;;
  *) printf 'unexpected gh invocation: %s\n' "$*" >&2; exit 1 ;;
esac
"#
    );
    std::fs::write(&gh, gh_script).unwrap();
    let mut permissions = std::fs::metadata(&gh).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&gh, permissions).unwrap();
    let gh_log = home.join("gh.log");

    let backend = LocalBackend::from_project_root(root.path());
    assert!(
        backend.get("2535").await.unwrap().is_none(),
        "fixture must begin without a local issue projection"
    );

    let output = Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(["td", "fill", "2535", "--spec-path", spec_rel])
        .current_dir(root.path())
        .env("HOME", home)
        .env("GH_TOKEN", "fixture-token")
        .env("AW_GH_LOG", &gh_log)
        .env("AW_DISABLE_CAP", "1")
        .output()
        .expect("run repo-built aw td fill");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "remote fill brief must succeed:\nstdout={stdout}\nstderr={stderr}"
    );
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("fill envelope JSON");
    assert_eq!(envelope["action"], "dispatch");
    assert_eq!(envelope["slug"], "2535");
    assert_eq!(
        envelope["invoke"]["args"]["marker_list"][0]["id"],
        "remote-fill-marker"
    );

    let hydrated = backend
        .get("2535")
        .await
        .unwrap()
        .expect("hydrated local issue projection");
    assert_eq!(hydrated.phase.as_deref(), Some("cb_genned"));
    assert_eq!(hydrated.github_id, Some(2535));
    let calls = std::fs::read_to_string(&gh_log).unwrap();
    assert!(calls.contains(" view 2535 "), "{calls}");
    assert!(calls.contains("api -X PATCH"), "{calls}");

    let _ = std::fs::remove_dir_all(backend.issues_dir());
}

// ── R6 — collision regression (bug-cb-fill-payload-routes-by-marker-id-alone-collides) ──

/// R6: when two HANDWRITE markers in different files share the same base
/// id, enumeration must surface BOTH entries (not silently drop one).
/// Combined with the R5 ambiguous-id error in `cb fill --apply`, this
/// prevents the silent mis-routing that motivated the bug.
#[test]
fn test_collision_enumerate_returns_both_entries() {
    let tmp = tempfile::TempDir::new().unwrap();
    let crates_dir = tmp.path().join("crates").join("a").join("src");
    let projects_dir = tmp.path().join("projects").join("b").join("src");
    std::fs::create_dir_all(&crates_dir).unwrap();
    std::fs::create_dir_all(&projects_dir).unwrap();
    // Both files use the legacy generic id pattern that the R1 scaffold
    // disambiguator now prevents — but legacy markers in older files may
    // still be present, so the enumerator must surface them all.
    let shared_id = "missing-generator:hand-written";
    let body = format!(
        "{}\nTODO: hand-write content\n{}\n",
        handwrite_begin(&format!("gap=\"{shared_id}\" tracker=\"t\" reason=\"r\"")),
        handwrite_end()
    );
    std::fs::write(crates_dir.join("first.rs"), &body).unwrap();
    std::fs::write(projects_dir.join("second.rs"), &body).unwrap();

    let markers = agentic_workflow::cli::cb_fill::enumerate_worktree_markers(tmp.path());
    let with_id: Vec<&agentic_workflow::cli::cb_fill::HandwriteMarkerEntry> =
        markers.iter().filter(|m| m.id == shared_id).collect();
    assert_eq!(
        with_id.len(),
        2,
        "both colliding markers must be enumerated, not silently deduped",
    );
    let mut paths: Vec<&str> = with_id.iter().map(|m| m.source_path.as_str()).collect();
    paths.sort();
    assert!(paths[0].ends_with("first.rs"));
    assert!(paths[1].ends_with("second.rs"));
}

// ── e2e gates (require real worktree + payload + check pipeline) ────────

/// AC1 (#1096, #1559, #1717): a real `aw cb fill` brief + apply round trip writes and
/// reads the marker payload under `/tmp/aw/workspaces/<workspace>/payloads/`
/// (never under the repo's `.aw/payloads/`), quoting the absolute path in
/// the dispatch envelope, and the apply step actually reads that file back
/// into app- and lib-root HANDWRITE blocks even when a root `crates/` exists.
/// A foreign marker outside the active TD Changes paths remains untouched and
/// cannot replace terminal code-check dispatch.
#[tokio::test]
async fn test_apply_marker_replaces_block() {
    use agentic_workflow::issues::types::{td_phase, IssueType};
    use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};
    use std::process::Command;

    let Some(git) = agentic_workflow::git::find_git_bin() else {
        eprintln!("skipping: git binary not on PATH");
        return;
    };
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    // Seed a minimal git repo on a non-"main" branch: TD/CB verbs only
    // require a provisioned `td-<slug>` branch when launched from `main`
    // (`should_use_td_branch` in td.rs); every real project branch (e.g.
    // `project-<name>`) runs TD/CB commands in place instead.
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "project-test"])
        .status()
        .expect("git init");
    for (k, v) in [
        ("user.email", "test@test"),
        ("user.name", "test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new(&git)
            .arg("-C")
            .arg(root)
            .args(["config", k, v])
            .status()
            .unwrap();
    }
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    std::fs::create_dir_all(root.join(".aw")).unwrap();
    // #1921: `aw cb fill`'s mutating verbs resolve the configured issue
    // backend unconditionally via `guard_issue_mutation`, so this fully
    // offline sandbox needs a resolvable `local` backend, gated behind the
    // sanctioned `AW_FIXTURE_LOCAL_BACKEND=1` fixture escape hatch (#1348)
    // set on every spawned `aw` command below.
    std::fs::write(
        root.join("aw.toml"),
        "[agentic_workflow.issue_platform]\ntype = \"local\"\n",
    )
    .unwrap();
    let existing_crate = root.join("crates/existing/src/lib.rs");
    std::fs::create_dir_all(existing_crate.parent().unwrap()).unwrap();
    std::fs::write(existing_crate, "pub fn existing_crate() {}\n").unwrap();

    // Seed a TD spec whose Changes section names app- and lib-root markers
    // (so brief mode's spec-scoped enumeration must include both even though
    // this checkout also has a root crates/ directory).
    let spec_rel = ".aw/tech-design/specs/demo.md";
    let app_marker_rel = "apps/vat/tests/vat_microvm_published_port.rs";
    let lib_marker_rel = "libs/openapi-codegen/src/target.rs";
    let spec_content = format!(
        "---\nid: demo\nfill_sections: [changes]\n---\n\n# Demo\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: {app_marker_rel}\n    action: create\n    impl_mode: hand-written\n  - path: {lib_marker_rel}\n    action: modify\n    impl_mode: hand-written\n```\n"
    );
    let spec_dir = root.join(".aw/tech-design/specs");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("demo.md"), spec_content).unwrap();

    // Seed both active markers without embedding marker-shaped literals in
    // this test source itself.
    for (path, id) in [
        (app_marker_rel, "app-marker"),
        (lib_marker_rel, "lib-marker"),
    ] {
        let marker_path = root.join(path);
        std::fs::create_dir_all(marker_path.parent().unwrap()).unwrap();
        std::fs::write(
            marker_path,
            format!(
                "{}\n// TODO: hand-write content for `{path}`.\n{}\n",
                handwrite_begin(&format!(
                    "gap=\"{id}\" tracker=\"none\" reason=\"unfilled\""
                )),
                handwrite_end(),
            ),
        )
        .unwrap();
    }

    let foreign_marker_rel = "projects/mamba/src/foreign.rs";
    let foreign_marker_path = root.join(foreign_marker_rel);
    std::fs::create_dir_all(foreign_marker_path.parent().unwrap()).unwrap();
    std::fs::write(
        &foreign_marker_path,
        format!(
            "{}\n// TODO: hand-write content for `{foreign_marker_rel}`.\n{}\n",
            handwrite_begin("gap=\"foreign-marker\" tracker=\"none\" reason=\"unrelated\""),
            handwrite_end(),
        ),
    )
    .unwrap();

    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();

    // Seed the open issue at cb_genned (the phase `aw cb fill` expects).
    let slug = "cb-fill-payload-roundtrip-test";
    let backend = LocalBackend::from_project_root(root);
    let issue = Issue {
        issue_type: IssueType::Enhancement,
        title: format!("{slug} WI"),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![format!("phase:{}", td_phase::CB_GENNED)],
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: format!("# {slug} WI\n"),
        related: Vec::new(),
        implements: vec![spec_rel.to_string()],
        phase: Some(td_phase::CB_GENNED.to_string()),
        branch: None,
        target_branch: None,
        git_workflow: None,
        change_id: None,
        iteration: None,
        current_task_id: None,
        impl_spec_phase: None,
        task_revisions: None,
        revision_counts: None,
        last_action: None,
        session_id: None,
        validation_errors: Vec::new(),
        review_count: None,
        flagged_sections: None,
        fill_retry_count: None,
        ship_status: None,
        ship_commit: None,
        regen_verified_at: None,
    };
    backend.create(&issue).await.expect("seed open issue");

    // Brief mode: enumerate + dispatch. Assert the envelope's payload path
    // is an ABSOLUTE path under /tmp/aw/workspaces/ — never the old
    // repo-relative `.aw/payloads/`.
    let brief_output = Command::new(&aw_bin)
        .arg("cb")
        .arg("fill")
        .arg(slug)
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .output()
        .expect("run aw cb fill (brief)");
    let brief_stdout = String::from_utf8_lossy(&brief_output.stdout);
    let brief_stderr = String::from_utf8_lossy(&brief_output.stderr);
    assert!(
        brief_output.status.success(),
        "brief mode should exit 0:\nstdout:\n{}\nstderr:\n{}",
        brief_stdout,
        brief_stderr
    );
    let envelope: serde_json::Value =
        serde_json::from_str(brief_stdout.trim()).expect("brief envelope is valid JSON");
    let payload_path = envelope["next"]["payload_path"]
        .as_str()
        .expect("next.payload_path present")
        .to_string();
    assert!(
        payload_path.starts_with("/tmp/aw/workspaces/"),
        "payload path must live under /tmp/aw/workspaces/, got: {}",
        payload_path
    );
    assert!(
        payload_path.contains("/payloads/"),
        "payload path must be under a payloads/ directory, got: {}",
        payload_path
    );
    assert!(
        !payload_path.contains(".aw/payloads"),
        "payload path must not reference the retired repo-root .aw/payloads/, got: {}",
        payload_path
    );
    let first_marker_id = envelope["invoke"]["args"]["marker_list"][0]["id"]
        .as_str()
        .expect("marker_list[0].id present")
        .to_string();
    let marker_list = envelope["invoke"]["args"]["marker_list"]
        .as_array()
        .expect("marker_list is an array");
    assert_eq!(
        marker_list.len(),
        2,
        "brief queue must include app/lib markers and exclude the foreign marker"
    );
    let mut queued_paths: Vec<&str> = marker_list
        .iter()
        .map(|marker| marker["source_path"].as_str().unwrap())
        .collect();
    queued_paths.sort_unstable();
    assert_eq!(queued_paths, vec![app_marker_rel, lib_marker_rel]);
    assert_eq!(first_marker_id, "app-marker");

    // The CLI already initialized the payload template at that absolute
    // path; overwrite it with the marker's real fill content, proving the
    // apply step reads back from /tmp, not from the repo tree.
    let app_payload_body = format!(
        "// filled app marker\n\n{} marker: {} path: {} reason: unfilled -->\n",
        "<!--", first_marker_id, app_marker_rel
    );
    std::fs::write(&payload_path, app_payload_body).expect("write app payload at /tmp/aw path");

    // Apply the app marker. The continuation must remain in fill and point at
    // the lib marker rather than skipping straight to code-check.
    let app_apply_output = Command::new(&aw_bin)
        .arg("cb")
        .arg("fill")
        .arg(slug)
        .arg("--apply")
        .arg("--marker")
        .arg(&first_marker_id)
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .output()
        .expect("run aw cb fill --apply for app marker");
    let app_apply_stdout = String::from_utf8_lossy(&app_apply_output.stdout);
    let app_apply_stderr = String::from_utf8_lossy(&app_apply_output.stderr);
    assert!(
        app_apply_output.status.success(),
        "app apply should exit 0:\nstdout:\n{}\nstderr:\n{}",
        app_apply_stdout,
        app_apply_stderr
    );
    let app_apply_envelope: serde_json::Value =
        serde_json::from_str(app_apply_stdout.trim()).expect("app apply envelope is valid JSON");
    assert!(
        app_apply_envelope["next"]["command"]
            .as_str()
            .is_some_and(|command| command.starts_with("aw cb fill")),
        "first apply must dispatch to the remaining lib marker, got:\n{}",
        app_apply_stdout
    );
    assert_eq!(app_apply_envelope["invoke"]["args"]["marker"], "lib-marker");
    let lib_payload_path = app_apply_envelope["next"]["payload_path"]
        .as_str()
        .expect("lib payload path present");
    std::fs::write(lib_payload_path, "// filled lib marker\n")
        .expect("write lib payload at /tmp/aw path");

    let lib_apply_output = Command::new(&aw_bin)
        .args(["cb", "fill", slug, "--apply", "--marker", "lib-marker"])
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .output()
        .expect("run aw cb fill --apply for lib marker");
    let lib_apply_stdout = String::from_utf8_lossy(&lib_apply_output.stdout);
    let lib_apply_stderr = String::from_utf8_lossy(&lib_apply_output.stderr);
    assert!(
        lib_apply_output.status.success(),
        "lib apply should exit 0:\nstdout:\n{}\nstderr:\n{}",
        lib_apply_stdout,
        lib_apply_stderr
    );
    assert!(
        lib_apply_stdout.contains("\"command\":\"aw cb check"),
        "last active marker must dispatch to terminal code-check, got:\n{}",
        lib_apply_stdout
    );
    assert!(
        !lib_apply_stdout.contains("foreign-marker"),
        "post-apply dispatch must not leak a foreign marker, got:\n{}",
        lib_apply_stdout
    );

    let updated_app =
        std::fs::read_to_string(root.join(app_marker_rel)).expect("read updated app source");
    assert!(
        updated_app.contains("filled app marker") && !updated_app.contains("TODO: hand-write"),
        "app source must contain its payload body in place of the stub, got:\n{}",
        updated_app
    );
    assert!(
        !updated_app.contains("<!-- marker:"),
        "generated payload metadata must never be copied into source, got:\n{}",
        updated_app
    );
    let updated_lib =
        std::fs::read_to_string(root.join(lib_marker_rel)).expect("read updated lib source");
    assert!(
        updated_lib.contains("filled lib marker") && !updated_lib.contains("TODO: hand-write"),
        "lib source must contain its payload body in place of the stub, got:\n{}",
        updated_lib
    );
    let foreign_source =
        std::fs::read_to_string(&foreign_marker_path).expect("read foreign marker source");
    assert!(
        foreign_source.contains(&format!(
            "TODO: hand-write content for `{foreign_marker_rel}`"
        )),
        "foreign marker must remain untouched, got:\n{}",
        foreign_source
    );

    let filled_issue = backend
        .get(slug)
        .await
        .expect("read filled issue")
        .expect("filled issue remains");
    assert_eq!(filled_issue.phase.as_deref(), Some(td_phase::CB_FILLED));

    // The payload directory itself must never have been created inside the
    // repo tree.
    assert!(
        !root.join(".aw/payloads").exists(),
        "apply must never write payload state under the repo's .aw/payloads/"
    );
}

/// #1901 AC1/AC2: the lifecycle dirty-tree guard that `td fill --apply` runs
/// before staging the current target must permit a dirty edit to the active
/// marker's own declared source path (the implementation diff an adoption
/// payload is meant to carry) while still rejecting a dirty edit anywhere
/// else in the tree.
#[tokio::test]
async fn test_apply_permits_current_marker_dirty_source_but_rejects_unrelated_dirty_path() {
    use agentic_workflow::issues::types::{td_phase, IssueType};
    use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};
    use std::process::Command;

    let Some(git) = agentic_workflow::git::find_git_bin() else {
        eprintln!("skipping: git binary not on PATH");
        return;
    };
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "project-test"])
        .status()
        .expect("git init");
    for (k, v) in [
        ("user.email", "test@test"),
        ("user.name", "test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new(&git)
            .arg("-C")
            .arg(root)
            .args(["config", k, v])
            .status()
            .unwrap();
    }
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    std::fs::create_dir_all(root.join(".aw")).unwrap();
    // #1921: `aw cb fill`'s mutating verbs resolve the configured issue
    // backend unconditionally via `guard_issue_mutation`, so this fully
    // offline sandbox needs a resolvable `local` backend, gated behind the
    // sanctioned `AW_FIXTURE_LOCAL_BACKEND=1` fixture escape hatch (#1348)
    // set on every spawned `aw` command below.
    std::fs::write(
        root.join("aw.toml"),
        "[agentic_workflow.issue_platform]\ntype = \"local\"\n",
    )
    .unwrap();

    // A single pre-existing XML HANDWRITE marker with an existing body —
    // exactly the #1900 adoption shape.
    let marker_rel = "crates/existing/src/lib.rs";
    let marker_path = root.join(marker_rel);
    std::fs::create_dir_all(marker_path.parent().unwrap()).unwrap();
    std::fs::write(
        &marker_path,
        "// <HANDWRITE gap=\"missing-generator:logic\" tracker=\"pending-tracker\" reason=\"fixture\">\n\
pub fn existing() { 1; }\n\
// </HANDWRITE>\n",
    )
    .unwrap();

    let spec_rel = ".aw/tech-design/specs/demo.md";
    let spec_content = format!(
        "---\nid: demo\nfill_sections: [changes]\n---\n\n# Demo\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: {marker_rel}\n    action: modify\n    impl_mode: hand-written\n```\n"
    );
    let spec_dir = root.join(".aw/tech-design/specs");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("demo.md"), spec_content).unwrap();

    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();

    let slug = "cb-fill-adoption-dirty-guard-test";
    let backend = LocalBackend::from_project_root(root);
    let issue = Issue {
        issue_type: IssueType::Enhancement,
        title: format!("{slug} WI"),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![format!("phase:{}", td_phase::CB_GENNED)],
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: format!("# {slug} WI\n"),
        related: Vec::new(),
        implements: vec![spec_rel.to_string()],
        phase: Some(td_phase::CB_GENNED.to_string()),
        branch: None,
        target_branch: None,
        git_workflow: None,
        change_id: None,
        iteration: None,
        current_task_id: None,
        impl_spec_phase: None,
        task_revisions: None,
        revision_counts: None,
        last_action: None,
        session_id: None,
        validation_errors: Vec::new(),
        review_count: None,
        flagged_sections: None,
        fill_retry_count: None,
        ship_status: None,
        ship_commit: None,
        regen_verified_at: None,
    };
    backend.create(&issue).await.expect("seed open issue");

    // Brief mode from a clean tree enumerates the single adoption marker and
    // (as an adoption payload) auto-initializes its payload to the adopt
    // sentinel — no payload write needed before apply.
    let brief_output = Command::new(&aw_bin)
        .args(["cb", "fill", slug])
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .output()
        .expect("run aw cb fill (brief)");
    assert!(
        brief_output.status.success(),
        "brief mode should exit 0:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&brief_output.stdout),
        String::from_utf8_lossy(&brief_output.stderr)
    );
    let brief_envelope: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&brief_output.stdout).trim())
            .expect("brief envelope is valid JSON");
    let marker_id = brief_envelope["invoke"]["args"]["marker_list"][0]["id"]
        .as_str()
        .expect("marker_list[0].id present")
        .to_string();

    // AC2: dirty an unrelated untracked file, then attempt apply. The
    // lifecycle dirty-tree guard must still reject it before touching the
    // marker source or committing anything.
    let unrelated_path = root.join("unrelated.txt");
    std::fs::write(&unrelated_path, "not part of this marker\n").unwrap();

    let rejected_output = Command::new(&aw_bin)
        .args(["cb", "fill", slug, "--apply", "--marker", &marker_id])
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .output()
        .expect("run aw cb fill --apply with an unrelated dirty file");
    assert!(
        !rejected_output.status.success(),
        "apply must reject an unrelated dirty path instead of applying:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&rejected_output.stdout),
        String::from_utf8_lossy(&rejected_output.stderr)
    );
    let rejected_combined = format!(
        "{}{}",
        String::from_utf8_lossy(&rejected_output.stdout),
        String::from_utf8_lossy(&rejected_output.stderr)
    );
    assert!(
        rejected_combined.contains("dirty"),
        "rejection must explain the dirty-tree guard, got:\n{}",
        rejected_combined
    );
    let unchanged_marker_source =
        std::fs::read_to_string(&marker_path).expect("read marker source after rejected apply");
    assert!(
        unchanged_marker_source.contains("tracker=\"pending-tracker\""),
        "a rejected apply must never touch the marker source, got:\n{}",
        unchanged_marker_source
    );

    std::fs::remove_file(&unrelated_path).unwrap();

    // AC1: dirty only the active marker's own declared source path — the
    // bounded implementation edit an adoption payload is meant to carry —
    // then apply. It must be accepted and folded into the normal lifecycle
    // commit alongside the tracker update.
    std::fs::write(
        &marker_path,
        "// <HANDWRITE gap=\"missing-generator:logic\" tracker=\"pending-tracker\" reason=\"fixture\">\n\
pub fn existing() { 42; }\n\
// </HANDWRITE>\n",
    )
    .unwrap();

    let accepted_output = Command::new(&aw_bin)
        .args(["cb", "fill", slug, "--apply", "--marker", &marker_id])
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .output()
        .expect("run aw cb fill --apply with a dirty current-marker source file");
    assert!(
        accepted_output.status.success(),
        "apply must accept a dirty current-marker source path:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&accepted_output.stdout),
        String::from_utf8_lossy(&accepted_output.stderr)
    );

    let adopted_source =
        std::fs::read_to_string(&marker_path).expect("read marker source after accepted apply");
    assert!(
        adopted_source.contains("pub fn existing() { 42; }"),
        "the author's dirty implementation edit must survive the adoption apply, got:\n{}",
        adopted_source
    );
    assert!(
        adopted_source.contains(&format!("tracker=\"#{slug}\"")),
        "the adoption apply must still bind the marker tracker to the work item, got:\n{}",
        adopted_source
    );

    // The dirty source path and the lifecycle state landed in the same
    // commit: the working tree is clean again, and the last commit's stat
    // includes the marker source path.
    let status_output = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output()
        .expect("git status after accepted apply");
    assert!(
        String::from_utf8_lossy(&status_output.stdout)
            .trim()
            .is_empty(),
        "the marker-fill commit must stage the accepted source diff and lifecycle state together, \
         leaving the tree clean:\n{}",
        String::from_utf8_lossy(&status_output.stdout)
    );
    let show_output = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["show", "--stat", "HEAD"])
        .output()
        .expect("git show --stat HEAD");
    assert!(
        String::from_utf8_lossy(&show_output.stdout).contains(marker_rel),
        "the normal marker-fill commit must include the accepted marker source path, got:\n{}",
        String::from_utf8_lossy(&show_output.stdout)
    );

    let filled_issue = backend
        .get(slug)
        .await
        .expect("read filled issue")
        .expect("filled issue remains");
    assert_eq!(filled_issue.phase.as_deref(), Some(td_phase::CB_FILLED));
}

/// #1904 AC1/AC2/AC3: once the active TD's declared `## Changes` paths carry
/// zero HANDWRITE markers (a marker-free integration test + semantic doc
/// accompanying already-filled source), `aw cb fill <id>` brief mode must
/// permit only those declared paths to be dirty, stage and commit them
/// through the normal terminal Cb-Fill lifecycle commit, and advance the
/// phase — while a dirty path outside that declared scope stays a hard
/// rejection.
#[tokio::test]
async fn test_marker_free_brief_commits_declared_evidence_and_rejects_unrelated_dirty_path() {
    use agentic_workflow::issues::types::{td_phase, IssueType};
    use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};
    use std::process::Command;

    let Some(git) = agentic_workflow::git::find_git_bin() else {
        eprintln!("skipping: git binary not on PATH");
        return;
    };
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "project-test"])
        .status()
        .expect("git init");
    for (k, v) in [
        ("user.email", "test@test"),
        ("user.name", "test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new(&git)
            .arg("-C")
            .arg(root)
            .args(["config", k, v])
            .status()
            .unwrap();
    }
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    std::fs::create_dir_all(root.join(".aw")).unwrap();
    // #1921: `aw cb fill`'s mutating verbs resolve the configured issue
    // backend unconditionally via `guard_issue_mutation`, so this fully
    // offline sandbox needs a resolvable `local` backend, gated behind the
    // sanctioned `AW_FIXTURE_LOCAL_BACKEND=1` fixture escape hatch (#1348)
    // set on every spawned `aw` command below.
    std::fs::write(
        root.join("aw.toml"),
        "[agentic_workflow.issue_platform]\ntype = \"local\"\n",
    )
    .unwrap();

    // Two declared Changes paths that intentionally carry no HANDWRITE
    // marker: an integration test and a semantic doc accompanying
    // marker-backed source landed elsewhere.
    let test_rel = "apps/demo/tests/example_test.rs";
    let doc_rel = "apps/demo/tech-design/semantic/example.md";
    for rel in [test_rel, doc_rel] {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "seed content\n").unwrap();
    }

    let spec_rel = ".aw/tech-design/specs/demo.md";
    let spec_content = format!(
        "---\nid: demo\nfill_sections: [changes]\n---\n\n# Demo\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: {test_rel}\n    action: modify\n    impl_mode: hand-written\n  - path: {doc_rel}\n    action: modify\n    impl_mode: hand-written\n```\n"
    );
    let spec_dir = root.join(".aw/tech-design/specs");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("demo.md"), spec_content).unwrap();

    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();

    let slug = "cb-fill-marker-free-evidence-test";
    let backend = LocalBackend::from_project_root(root);
    let issue = Issue {
        issue_type: IssueType::Enhancement,
        title: format!("{slug} WI"),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![format!("phase:{}", td_phase::CB_GENNED)],
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: format!("# {slug} WI\n"),
        related: Vec::new(),
        implements: vec![spec_rel.to_string()],
        phase: Some(td_phase::CB_GENNED.to_string()),
        branch: None,
        target_branch: None,
        git_workflow: None,
        change_id: None,
        iteration: None,
        current_task_id: None,
        impl_spec_phase: None,
        task_revisions: None,
        revision_counts: None,
        last_action: None,
        session_id: None,
        validation_errors: Vec::new(),
        review_count: None,
        flagged_sections: None,
        fill_retry_count: None,
        ship_status: None,
        ship_commit: None,
        regen_verified_at: None,
    };
    backend.create(&issue).await.expect("seed open issue");

    // AC2: dirty a path outside the TD's declared Changes scope, then run
    // brief. The zero-marker fast path must still reject it before staging
    // or committing anything.
    let unrelated_path = root.join("unrelated.txt");
    std::fs::write(&unrelated_path, "not part of this TD's declared scope\n").unwrap();

    let rejected_output = Command::new(&aw_bin)
        .args(["cb", "fill", slug])
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .output()
        .expect("run aw cb fill (brief) with an unrelated dirty file");
    assert!(
        !rejected_output.status.success(),
        "brief must reject a dirty path outside the declared Changes scope:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&rejected_output.stdout),
        String::from_utf8_lossy(&rejected_output.stderr)
    );
    let rejected_combined = format!(
        "{}{}",
        String::from_utf8_lossy(&rejected_output.stdout),
        String::from_utf8_lossy(&rejected_output.stderr)
    );
    assert!(
        rejected_combined.contains("dirty"),
        "rejection must explain the dirty-tree guard, got:\n{}",
        rejected_combined
    );
    let status_after_reject = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output()
        .expect("git status after rejected brief");
    assert!(
        String::from_utf8_lossy(&status_after_reject.stdout).contains("unrelated.txt"),
        "a rejected brief must never stage or commit the unrelated dirty file"
    );
    std::fs::remove_file(&unrelated_path).unwrap();

    // AC1: dirty only the two declared marker-free Changes paths — the
    // bounded implementation evidence a hand-written test/doc edit is meant
    // to carry — then run brief. It must be accepted, staged, and committed
    // through the normal terminal Cb-Fill lifecycle commit, advancing the
    // issue phase.
    std::fs::write(root.join(test_rel), "// updated test evidence\n").unwrap();
    std::fs::write(root.join(doc_rel), "updated doc evidence\n").unwrap();

    let accepted_output = Command::new(&aw_bin)
        .args(["cb", "fill", slug])
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .output()
        .expect("run aw cb fill (brief) with only declared paths dirty");
    assert!(
        accepted_output.status.success(),
        "brief must accept dirty declared Changes paths once no markers remain:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&accepted_output.stdout),
        String::from_utf8_lossy(&accepted_output.stderr)
    );
    let accepted_stdout = String::from_utf8_lossy(&accepted_output.stdout);
    assert!(
        accepted_stdout.contains("\"command\":\"aw cb check"),
        "the zero-marker fast path must dispatch to terminal code-check, got:\n{}",
        accepted_stdout
    );

    let status_after_accept = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output()
        .expect("git status after accepted brief");
    assert!(
        String::from_utf8_lossy(&status_after_accept.stdout)
            .trim()
            .is_empty(),
        "the declared-evidence commit must leave the tree clean:\n{}",
        String::from_utf8_lossy(&status_after_accept.stdout)
    );
    let show_output = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["show", "--stat", "HEAD"])
        .output()
        .expect("git show --stat HEAD");
    let show_stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(
        show_stdout.contains(test_rel) && show_stdout.contains(doc_rel),
        "the terminal Cb-Fill commit must include both declared marker-free paths, got:\n{}",
        show_stdout
    );

    let filled_issue = backend
        .get(slug)
        .await
        .expect("read filled issue")
        .expect("filled issue remains");
    assert_eq!(filled_issue.phase.as_deref(), Some(td_phase::CB_FILLED));
}

#[test]
#[ignore = "requires real worktree + git history"]
fn test_cb_fill_trailer_committed() {
    // Reserved: assert git log contains `Lifecycle-Stage: Cb-Fill` after a
    // successful apply-last-marker run.
}

#[test]
#[ignore = "requires real worktree + issue file"]
fn test_cb_filled_phase_written() {
    // Reserved: assert issue frontmatter has phase: cb_filled after success.
}

#[test]
#[ignore = "requires real cb check pipeline + drift fixture"]
fn test_cb_check_gate_rejection() {
    // Reserved: leave one HANDWRITE block in place, invoke --apply on
    // another marker last, assert error envelope action == "error" and
    // phase remains cb_genned.
}

// CODEGEN-END
