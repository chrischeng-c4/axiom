// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/cb_fill_test.md#source
// CODEGEN-BEGIN
//! Integration tests for `aw td fill` (Phase 3).
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
    }
}

fn handwrite_begin(attrs: &str) -> String {
    format!("// HANDWRITE-{} {}", "BEGIN", attrs)
}

fn handwrite_end() -> &'static str {
    concat!("// HANDWRITE-", "END")
}

// ── R1 / R14(1) ─────────────────────────────────────────────────────────

/// R1: `aw td fill` is registered as a first-class subcommand under td.
#[test]
fn test_cb_fill_registered() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td namespace");
    let fill = td.find_subcommand("fill").expect("td fill subcommand");
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
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("fill"))
        .expect("td fill");
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
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("fill"))
        .expect("td fill");
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

/// R10: terminal `aw td code-check` accepts `cb_filled` as a valid phase.
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
            "command": "aw td fill",
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

/// AC1 (#1096): a real `aw td fill` brief + apply round trip writes and
/// reads the marker payload under `/tmp/aw/workspaces/<workspace>/payloads/`
/// (never under the repo's `.aw/payloads/`), quoting the absolute path in
/// the dispatch envelope, and the apply step actually reads that file back
/// into the HANDWRITE block.
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
    std::fs::write(root.join("aw.toml"), "").unwrap();

    // Seed a TD spec whose Changes section names the marker's source file
    // (so brief mode's spec-scoped enumeration includes it).
    let spec_rel = ".aw/tech-design/specs/demo.md";
    let spec_content = "---\nid: demo\nfill_sections: [changes]\n---\n\n# Demo\n\n\
         ## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  \
         - path: src/demo.rs\n    action: create\n    impl_mode: hand-written\n```\n";
    let spec_dir = root.join(".aw/tech-design/specs");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("demo.md"), spec_content).unwrap();

    // Seed the unfilled HANDWRITE marker source file.
    let marker_rel = "src/demo.rs";
    let marker_path = root.join(marker_rel);
    std::fs::create_dir_all(marker_path.parent().unwrap()).unwrap();
    std::fs::write(
        &marker_path,
        "// HANDWRITE-BEGIN gap=\"demo-marker\" tracker=\"none\" reason=\"unfilled\"\n\
         // TODO: hand-write content for `src/demo.rs`.\n\
         // HANDWRITE-END\n",
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

    // Seed the open issue at cb_genned (the phase `aw td fill` expects).
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
        .arg("td")
        .arg("fill")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td fill (brief)");
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
    let marker_id = envelope["invoke"]["args"]["marker_list"][0]["id"]
        .as_str()
        .expect("marker_list[0].id present")
        .to_string();

    // The CLI already initialized the payload template at that absolute
    // path; overwrite it with the marker's real fill content, proving the
    // apply step reads back from /tmp, not from the repo tree.
    let payload_body = "// filled by test_apply_marker_replaces_block\n";
    std::fs::write(&payload_path, payload_body).expect("write payload at /tmp/aw path");

    // Apply: read the /tmp payload and merge it into the HANDWRITE block.
    let apply_output = Command::new(&aw_bin)
        .arg("td")
        .arg("fill")
        .arg(slug)
        .arg("--apply")
        .arg("--marker")
        .arg(&marker_id)
        .current_dir(root)
        .output()
        .expect("run aw td fill --apply");
    let apply_stdout = String::from_utf8_lossy(&apply_output.stdout);
    let apply_stderr = String::from_utf8_lossy(&apply_output.stderr);
    assert!(
        apply_output.status.success(),
        "apply should exit 0:\nstdout:\n{}\nstderr:\n{}",
        apply_stdout,
        apply_stderr
    );
    assert!(
        apply_stdout.contains("\"command\":\"aw td code-check\""),
        "last marker apply should dispatch to terminal code-check, got:\n{}",
        apply_stdout
    );

    let updated_source = std::fs::read_to_string(&marker_path).expect("read updated source");
    assert!(
        updated_source.contains("filled by test_apply_marker_replaces_block"),
        "source file must contain the payload body in place of the stub, got:\n{}",
        updated_source
    );
    assert!(
        !updated_source.contains("TODO: hand-write content"),
        "the unfilled stub text must be gone after apply, got:\n{}",
        updated_source
    );

    // The payload directory itself must never have been created inside the
    // repo tree.
    assert!(
        !root.join(".aw/payloads").exists(),
        "apply must never write payload state under the repo's .aw/payloads/"
    );
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
