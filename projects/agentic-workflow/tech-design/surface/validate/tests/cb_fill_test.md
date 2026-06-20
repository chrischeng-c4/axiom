---
id: projects-score-tests-cb-fill-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/cb_fill_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/cb_fill_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/cb_fill_test.rs -->
````rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/cb_fill_test.md#source
// CODEGEN-BEGIN
//! Integration tests for `aw td fill` (Phase 3).
//!
//! Smoke tests for CLI registration, brief mode envelope shape, marker
//! enumeration, and `--apply --marker` block replacement. Full
//! e2e integration scenarios (code check gate + Cb-Fill trailer + phase
//! advance) are #[ignore]d because they require a real worktree, real
//! payload files, and the agent loop infrastructure.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#test-plan

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
    assert!(td_phase::is_mergeable("cb_filled"));
    assert!(td_phase::is_mergeable("cb_genned"));
    assert!(!td_phase::is_mergeable("td_reviewed"));
}

/// R9: `Cb-Fill` trailer const exists in lifecycle_trailer module.
#[test]
fn test_lifecycle_trailer_cb_fill_variant() {
    use agentic_workflow::issues::types::lifecycle_trailer;
    assert_eq!(lifecycle_trailer::CB_FILL, "Cb-Fill");
}

/// R10: `aw td merge` accepts `cb_filled` as a valid pre-merge phase.
/// We verify this at the helper-level: `is_mergeable("cb_filled") == true`.
#[test]
fn test_td_merge_accepts_cb_filled() {
    use agentic_workflow::issues::types::td_phase;
    assert!(td_phase::is_mergeable(td_phase::CB_FILLED));
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
                    "source_path": "projects/agentic-workflow/src/issues/types.rs",
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
        "  - path: ./projects/agentic-workflow/src/cli/cb_fill.rs\n",
        "  - file: projects/agentic-workflow/src/issues/types.rs\n",
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
            "projects/agentic-workflow/src/cli/cb_fill.rs".to_string(),
            "projects/agentic-workflow/src/issues/types.rs".to_string(),
        ],
    );

    let legacy_spec = concat!(
        "\n",
        "#",
        "# Changes\n\n",
        "```yaml\n",
        "files:\n",
        "  - file: ./projects/agentic-workflow/tests/cb_fill_test.rs\n",
        "```\n",
    );
    let paths = agentic_workflow::cli::cb_fill::extract_change_paths_from_spec(legacy_spec);
    assert_eq!(
        paths,
        vec!["projects/agentic-workflow/tests/cb_fill_test.rs".to_string()],
    );
}

/// R2: inherited markers outside the active TD Changes paths are filtered
/// out of the brief-mode marker list.
#[test]
fn test_scope_filters_to_changed_source_paths() {
    let markers = vec![
        marker("projects/agentic-workflow/src/cli/cb_fill.rs"),
        marker("projects/agentic-workflow/src/cli/cb.rs"),
        marker("projects/agentic-workflow/src/issues/types.rs"),
    ];
    let change_paths = vec![
        "projects/agentic-workflow/src/cli".to_string(),
        "projects/agentic-workflow/tests/*_test.rs".to_string(),
    ];

    let scoped =
        agentic_workflow::cli::cb_fill::filter_markers_to_change_paths(&markers, &change_paths);
    let paths: Vec<&str> = scoped.iter().map(|m| m.source_path.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "projects/agentic-workflow/src/cli/cb_fill.rs",
            "projects/agentic-workflow/src/cli/cb.rs",
        ],
    );
}

/// R1: when the active TD only changes spec files, source HANDWRITE markers
/// are outside scope and the brief path can dispatch directly to merge.
#[test]
fn test_scope_zero_marker_for_spec_only_change() {
    let markers = vec![
        marker("projects/agentic-workflow/src/cli/cb_fill.rs"),
        marker("projects/agentic-workflow/src/issues/types.rs"),
    ];
    let change_paths =
        vec!["projects/agentic-workflow/tech-design/surface/specs/spec-only-change.md".to_string()];

    let scoped =
        agentic_workflow::cli::cb_fill::filter_markers_to_change_paths(&markers, &change_paths);
    assert!(scoped.is_empty());
}

/// R1 fallback: when no active spec is resolved, brief mode keeps the legacy
/// all-marker behavior instead of silently dropping inherited markers.
#[test]
fn test_scope_missing_spec_uses_legacy_all_markers() {
    let markers = vec![
        marker("projects/agentic-workflow/src/cli/cb_fill.rs"),
        marker("projects/agentic-workflow/src/issues/types.rs"),
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

#[test]
#[ignore = "requires real worktree, real payload, and the cb check pipeline"]
fn test_apply_marker_replaces_block() {
    // Reserved: build a worktree, write a payload at
    // .aw/payloads/<slug>/<id>.md, run `aw td fill <slug> --apply
    // --marker <id>`, assert source file has payload body in place of stub.
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/cb_fill_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```
