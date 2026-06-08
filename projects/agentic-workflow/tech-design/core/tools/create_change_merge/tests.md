---
id: sdd-tools-create-change-merge-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change merge tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_merge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 69 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod workflow_tests {
    use super::test_support::setup_change;
    use super::*;

    #[tokio::test]
    async fn test_programmatic_merge_with_main_spec_ref() {
        let tmp = setup_change("pm-test", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-test");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // merge_strategy is dead code — no longer stripped, so omit from input
        let spec_content = "---\nid: auth-flow\nmain_spec_ref: sdd/workflow/auth-flow.md\ncreate_complete: true\nfill_sections: [overview]\nfilled_sections: [overview]\n---\n\n# Auth Flow\n\n## Overview\n\nAuth flow spec.\n\n# Reviews\n\n## Review: Test\nAll good.\n";
        std::fs::write(change_dir.join("specs/auth-flow.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["specs_merged"].as_array().unwrap().len(), 1);
        assert_eq!(
            parsed["specs_merged"][0]["target"],
            ".aw/tech-design/sdd/workflow/auth-flow.md"
        );

        // Verify target file exists and change-spec-only fields are stripped
        let target = tmp.path().join(".aw/tech-design/sdd/workflow/auth-flow.md");
        assert!(target.exists());
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(!content.contains("fill_sections"));
        assert!(!content.contains("filled_sections"));
        assert!(!content.contains("create_complete"));
        assert!(!content.contains("# Reviews"));
        assert!(content.contains("Auth flow spec."));

        // Verify change was moved to archive
        assert!(
            !change_dir.exists(),
            "change_dir should be moved to archive"
        );
        let archive_dir = tmp.path().join(parsed["archive_path"].as_str().unwrap());
        assert!(archive_dir.exists(), "archive dir should exist");

        // Archived phase lives in the closed issue (single source of truth under R4/R7).
        let closed_issue = crate::shared::workspace::issues_path(tmp.path())
            .join("closed")
            .join("pm-test.md");
        assert!(
            closed_issue.exists(),
            "closed issue must exist after archive"
        );
        let issue_body = std::fs::read_to_string(&closed_issue).unwrap();
        assert!(
            issue_body.contains("phase: change_archived"),
            "closed issue must record phase: change_archived:\n{}",
            issue_body
        );
    }

    #[tokio::test]
    async fn test_missing_main_spec_ref_rejected() {
        // Null main_spec_ref is now a hard error — no fallback to spec_id.md.
        let tmp = setup_change("pm-noref", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-noref");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let spec_content =
            "---\nid: new-feature\nmain_spec_ref: ~\n---\n\n# New Feature\n\n## Overview\n\nNew.\n";
        std::fs::write(change_dir.join("specs/new-feature.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-noref"
        });
        let result = execute_workflow(&args, tmp.path()).await;
        assert!(result.is_err(), "null main_spec_ref must be a hard error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("main_spec_ref"),
            "error must mention main_spec_ref: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_root_level_path_rejected() {
        // main_spec_ref without '/' is a hard error — merge aborted, no files written.
        let tmp = setup_change("pm-rootpath", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-rootpath");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let spec_content =
            "---\nid: flat-spec\nmain_spec_ref: flat-spec.md\n---\n\n# Flat\n\nContent.\n";
        std::fs::write(change_dir.join("specs/flat-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-rootpath"
        });
        let result = execute_workflow(&args, tmp.path()).await;
        assert!(result.is_err(), "root-level path must return a hard error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("root-level") || err.contains("subfolder"),
            "error must mention root-level restriction: {}",
            err
        );

        // No target file should have been written
        let flat_target = tmp.path().join(".aw/tech-design/flat-spec.md");
        assert!(
            !flat_target.exists(),
            "no target file should be written on hard error"
        );
    }

    #[tokio::test]
    async fn test_audit_log_create() {
        // When target does not exist, audit_log must contain "audit: create .aw/tech-design/..."
        let tmp = setup_change("pm-audit-create", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-audit-create");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let spec_content = "---\nid: new-spec\nmain_spec_ref: sdd/logic/new-spec.md\n---\n\n# New Spec\n\nContent.\n";
        std::fs::write(change_dir.join("specs/new-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-audit-create"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let audit_log = parsed["audit_log"].as_array().unwrap();
        assert_eq!(audit_log.len(), 1);
        assert_eq!(
            audit_log[0].as_str().unwrap(),
            "[merge] create sdd/logic/new-spec.md"
        );
    }

    #[tokio::test]
    async fn test_audit_log_section_merge() {
        // When target already exists (no .base.md), audit_log must contain "[merge] section-merge {path}"
        // REQ: bug-create-change-merge-archive-moves-not-committed-sp (defect 2)
        let tmp = setup_change(
            "pm-audit-section-merge",
            StatePhase::ChangeImplementationReviewed,
        );
        let change_dir = tmp.path().join(".aw/changes/pm-audit-section-merge");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Pre-create the target file with existing sections
        let target_dir = tmp.path().join(".aw/tech-design/sdd/logic");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(
            target_dir.join("existing-spec.md"),
            "---\nid: existing-spec\nmain_spec_ref: sdd/logic/existing-spec.md\n---\n\n# Existing Spec\n\n## Overview\n\nOld overview.\n\n## Details\n\nOld details.\n",
        ).unwrap();

        // Change spec only touches Overview, not Details
        let spec_content = "---\nid: existing-spec\nmain_spec_ref: sdd/logic/existing-spec.md\n---\n\n# Existing Spec\n\n## Overview\n\nNew overview content.\n";
        std::fs::write(change_dir.join("specs/existing-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-audit-section-merge"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let audit_log = parsed["audit_log"].as_array().unwrap();
        assert!(
            !audit_log.is_empty(),
            "audit_log must have at least one entry"
        );
        assert_eq!(
            audit_log[0].as_str().unwrap(),
            "[merge] section-merge sdd/logic/existing-spec.md"
        );

        // New content must be present, AND old untouched sections must be preserved
        let content = std::fs::read_to_string(target_dir.join("existing-spec.md")).unwrap();
        assert!(
            content.contains("New overview content."),
            "updated section must be present"
        );
        assert!(
            content.contains("Old details."),
            "untouched section must be preserved"
        );
        assert!(
            !content.contains("Old overview."),
            "updated section must replace old content"
        );
    }

    #[tokio::test]
    async fn test_validation_aborts_before_write() {
        // When any spec fails path validation, NO files must be written (all-or-nothing).
        let tmp = setup_change("pm-abort", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-abort");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Valid spec — listed first alphabetically
        let valid_spec = "---\nid: valid-spec\nmain_spec_ref: sdd/valid/valid-spec.md\n---\n\n# Valid\n\nContent.\n";
        std::fs::write(change_dir.join("specs/a-valid-spec.md"), valid_spec).unwrap();

        // Invalid spec (root-level path, no '/') — listed second
        let invalid_spec =
            "---\nid: root-spec\nmain_spec_ref: root-spec.md\n---\n\n# Root\n\nContent.\n";
        std::fs::write(change_dir.join("specs/b-root-spec.md"), invalid_spec).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-abort"
        });
        let result = execute_workflow(&args, tmp.path()).await;
        assert!(
            result.is_err(),
            "merge must fail when any spec has invalid path"
        );

        // The valid spec's target must NOT have been written — validation aborts before write pass
        let valid_target = tmp
            .path()
            .join(".aw/tech-design/sdd/valid/valid-spec.md");
        assert!(
            !valid_target.exists(),
            "no files should be written when validation aborts the merge"
        );
    }

    #[tokio::test]
    async fn test_3way_merge_clean() {
        // Setup: base snapshot + diverged main spec + change spec → clean 3-way merge
        // Changes are in non-overlapping regions so merge is clean.
        if find_git_binary().is_none() {
            // Skip: git is required for 3-way merge test
            return;
        }

        let tmp = setup_change("pm-3way-clean", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-3way-clean");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Base content (snapshot at change-init time) — has two sections
        let base_content = "\
    ---\nid: merge-test\nmain_spec_ref: sdd/logic/merge-test.md\n---\n\n\
    # Merge Test\n\n\
    ## Section One\n\n\
    Original section one.\n\n\
    ## Section Two\n\n\
    Original section two.\n";

        // Main spec on disk (diverged: modified Section One only)
        let main_content = "\
    ---\nid: merge-test\nmain_spec_ref: sdd/logic/merge-test.md\n---\n\n\
    # Merge Test\n\n\
    ## Section One\n\n\
    Main modified section one.\n\n\
    ## Section Two\n\n\
    Original section two.\n";

        // Change spec (modified Section Two only — non-overlapping with main's change)
        let change_content = "\
    ---\nid: merge-test\nmain_spec_ref: sdd/logic/merge-test.md\n\
    create_complete: true\nfill_sections: [overview]\nfilled_sections: [overview]\n---\n\n\
    # Merge Test\n\n\
    ## Section One\n\n\
    Original section one.\n\n\
    ## Section Two\n\n\
    Change updated section two.\n";

        // Write base snapshot
        std::fs::write(change_dir.join("specs/merge-test.base.md"), base_content).unwrap();

        // Write change spec
        std::fs::write(change_dir.join("specs/merge-test.md"), change_content).unwrap();

        // Write diverged main spec
        let target_dir = tmp.path().join(".aw/tech-design/sdd/logic");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("merge-test.md"), main_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-3way-clean"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Verify the merged file contains both non-overlapping changes:
        // main's modification to section one AND change spec's update to section two
        let merged = std::fs::read_to_string(target_dir.join("merge-test.md")).unwrap();
        assert!(
            merged.contains("Main modified section one."),
            "merged result must preserve main's diverged section one"
        );
        assert!(
            merged.contains("Change updated section two."),
            "merged result must include change spec's updated section two"
        );
        // Neither original line should remain
        assert!(
            !merged.contains("Original section one."),
            "original section one must be replaced by main's change"
        );
        assert!(
            !merged.contains("Original section two."),
            "original section two must be replaced by change spec's update"
        );
    }

    #[tokio::test]
    async fn test_3way_merge_conflict() {
        // Setup: both main and change spec modify the same line → conflict
        if find_git_binary().is_none() {
            // Skip: git is required for 3-way merge conflict test
            return;
        }

        let tmp = setup_change("pm-3way-conflict", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-3way-conflict");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Base content
        let base_content = "---\nid: conflict-test\nmain_spec_ref: sdd/logic/conflict-test.md\n---\n\n# Conflict Test\n\n## Overview\n\nOriginal line here.\n";

        // Main spec: same line changed to something different
        let main_content = "---\nid: conflict-test\nmain_spec_ref: sdd/logic/conflict-test.md\n---\n\n# Conflict Test\n\n## Overview\n\nMain changed this line to version A.\n";

        // Change spec: same line changed to something else
        let change_content = "---\nid: conflict-test\nmain_spec_ref: sdd/logic/conflict-test.md\n---\n\n# Conflict Test\n\n## Overview\n\nChange modified this line to version B.\n";

        std::fs::write(change_dir.join("specs/conflict-test.base.md"), base_content).unwrap();
        std::fs::write(change_dir.join("specs/conflict-test.md"), change_content).unwrap();

        let target_dir = tmp.path().join(".aw/tech-design/sdd/logic");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("conflict-test.md"), main_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-3way-conflict"
        });
        let result = execute_workflow(&args, tmp.path()).await;
        assert!(
            result.is_err(),
            "3-way merge with conflicts must return an error"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("conflict"),
            "error must mention conflict: {}",
            err
        );

        // Verify the main spec was NOT overwritten (all-or-nothing)
        let content = std::fs::read_to_string(target_dir.join("conflict-test.md")).unwrap();
        assert!(
            content.contains("Main changed this line to version A."),
            "main spec must remain unchanged after conflict abort"
        );
    }

    #[tokio::test]
    async fn test_base_md_skipped_by_find_specs() {
        // Verify that .base.md files are not included in find_specs_to_merge() results
        let tmp = setup_change("pm-skip-base", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-skip-base");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        // Write a regular spec and its .base.md companion
        let spec_content =
            "---\nid: some-spec\nmain_spec_ref: sdd/logic/some-spec.md\n---\n\n# Some Spec\n";
        let base_content = "---\nid: some-spec\nmain_spec_ref: sdd/logic/some-spec.md\n---\n\n# Some Spec (base)\n";
        std::fs::write(specs_dir.join("some-spec.md"), spec_content).unwrap();
        std::fs::write(specs_dir.join("some-spec.base.md"), base_content).unwrap();

        let found = helpers::find_specs_to_merge(&change_dir);
        assert_eq!(found.len(), 1, "only the regular spec should be found");
        let found_name = found[0].file_name().unwrap().to_str().unwrap();
        assert_eq!(
            found_name, "some-spec.md",
            "found file must be the regular spec"
        );
        assert!(
            !found
                .iter()
                .any(|p| p.to_str().unwrap().contains(".base.md")),
            ".base.md files must not appear in find_specs_to_merge results"
        );
    }

    #[tokio::test]
    async fn test_no_base_fallback_section_merge() {
        // Verify specs without .base.md use section-merge behavior when target exists
        // REQ: bug-create-change-merge-archive-moves-not-committed-sp (defect 2)
        let tmp = setup_change("pm-no-base", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-no-base");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Pre-create target file with existing sections
        let target_dir = tmp.path().join(".aw/tech-design/sdd/logic");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(
            target_dir.join("no-base-spec.md"),
            "---\nid: no-base-spec\nmain_spec_ref: sdd/logic/no-base-spec.md\n---\n\n# No Base Spec\n\n## Overview\n\nOriginal overview.\n\n## Details\n\nOriginal details.\n\n## History\n\nOriginal history.\n",
        ).unwrap();

        // Change spec without .base.md companion, modifies Overview and adds a new section
        let spec_content = "---\nid: no-base-spec\nmain_spec_ref: sdd/logic/no-base-spec.md\n---\n\n# No Base Spec\n\n## Overview\n\nNew content via section-merge.\n\n## New Section\n\nBrand new section.\n";
        std::fs::write(change_dir.join("specs/no-base-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-no-base"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Audit log must say "section-merge", not "overwrite" or "3way-merge"
        let audit_log = parsed["audit_log"].as_array().unwrap();
        assert!(
            !audit_log.is_empty(),
            "audit_log must have at least one entry"
        );
        let entry = audit_log[0].as_str().unwrap();
        assert!(
            entry.contains("section-merge"),
            "audit must record section-merge action when no .base.md and target exists: {}",
            entry
        );

        // Verify section-level merge: changed section updated, untouched sections preserved, new section added
        let content = std::fs::read_to_string(target_dir.join("no-base-spec.md")).unwrap();
        assert!(
            content.contains("New content via section-merge."),
            "updated section must contain new content"
        );
        assert!(
            !content.contains("Original overview."),
            "updated section must not contain old content"
        );
        assert!(
            content.contains("Original details."),
            "untouched Details section must be preserved"
        );
        assert!(
            content.contains("Original history."),
            "untouched History section must be preserved"
        );
        assert!(
            content.contains("Brand new section."),
            "new section must be appended"
        );
    }

    #[tokio::test]
    async fn test_audit_log_3way_merge() {
        // Verify audit log records '3way-merge' action for successful 3-way merges
        if find_git_binary().is_none() {
            // Skip: git is required for 3-way merge audit test
            return;
        }

        let tmp = setup_change("pm-audit-3way", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-audit-3way");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Use identical content for base and main (trivial merge — theirs wins cleanly)
        let base_and_main = "---\nid: audit-3way\nmain_spec_ref: sdd/logic/audit-3way.md\n---\n\n# Audit 3Way\n\n## Overview\n\nOriginal.\n";
        let change_content = "---\nid: audit-3way\nmain_spec_ref: sdd/logic/audit-3way.md\ncreate_complete: true\nfill_sections: [overview]\nfilled_sections: [overview]\n---\n\n# Audit 3Way\n\n## Overview\n\nUpdated by change.\n";

        std::fs::write(change_dir.join("specs/audit-3way.base.md"), base_and_main).unwrap();
        std::fs::write(change_dir.join("specs/audit-3way.md"), change_content).unwrap();

        let target_dir = tmp.path().join(".aw/tech-design/sdd/logic");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("audit-3way.md"), base_and_main).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-audit-3way"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let audit_log = parsed["audit_log"].as_array().unwrap();
        assert!(
            !audit_log.is_empty(),
            "audit log must have entries for 3-way merge"
        );
        let entry = audit_log[0].as_str().unwrap();
        assert_eq!(
            entry, "[merge] 3way-merge sdd/logic/audit-3way.md",
            "audit log must record 3way-merge action"
        );
    }

    // REQ: worktree-per-change — merge moves the associated issue to closed/
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_merge_closes_open_issue() {
        let tmp = setup_change(
            "enhancement-merge-closes",
            StatePhase::ChangeImplementationReviewed,
        );
        let change_dir = tmp.path().join(".aw/changes/enhancement-merge-closes");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Create a matching open issue
        let open_dir = crate::shared::workspace::issues_path(tmp.path()).join("open");
        std::fs::create_dir_all(&open_dir).unwrap();
        std::fs::write(
            open_dir.join("enhancement-merge-closes.md"),
            "---\ntype: enhancement\ntitle: Test merge closes issue\nstate: open\nphase: change_inited\nbranch: cclab/enhancement-merge-closes\ngit_workflow: worktree\n---\n\n## Problem\n\nBody.\n",
        )
        .unwrap();

        // Minimal valid spec
        let spec_content = "---\nid: some-spec\nmain_spec_ref: sdd/logic/some-spec.md\n---\n\n# Some Spec\n\nContent.\n";
        std::fs::write(change_dir.join("specs/some-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "enhancement-merge-closes"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(
            parsed["issue_closed"], true,
            "issue_closed flag should be set"
        );

        // File moved from open/ to closed/
        assert!(
            !open_dir.join("enhancement-merge-closes.md").exists(),
            "open issue file should be moved"
        );
        let closed_path = crate::shared::workspace::issues_path(tmp.path())
            .join("closed")
            .join("enhancement-merge-closes.md");
        assert!(closed_path.exists(), "closed issue file should exist");

        // REQ: R7 — state: closed, phase: change_archived, branch preserved
        let content = std::fs::read_to_string(&closed_path).unwrap();
        assert!(content.contains("state: closed"));
        assert!(
            content.contains("phase: change_archived"),
            "closed issue should have phase: change_archived:\n{}",
            content
        );
        assert!(
            content.contains("branch:"),
            "closed issue should retain branch field for audit trail:\n{}",
            content
        );
    }

    // Obsolete under R1: test_merge_without_issue_returns_false exercised the
    // "change has no backing issue" scenario. R1 of
    // refactor-eliminate-state-yaml-user-input-md-groups-nesting enforces
    // `change_id == issue_slug`, making this state unreachable — save()
    // would fail long before merge. Kept as a marker so future contributors
    // don't re-introduce the fallback.

    #[tokio::test]
    async fn test_programmatic_merge_no_specs() {
        let tmp = setup_change("pm-empty", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-empty");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // No spec files

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-empty"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(parsed["message"]
            .as_str()
            .unwrap()
            .contains("No specs to merge"));
        // Verify change was moved to archive
        assert!(!change_dir.exists());
    }
}

#[cfg(test)]
mod preflight_worktree_tests {
    use super::*;
    use crate::state::StateManager;
    use tempfile::TempDir;

    // REQ: change-merge R6 — pre-flight G1 aborts on dirty worktree.
    // Creates a real git repo with a worktree, dirties it with an uncommitted
    // file, and verifies execute_workflow aborts with actionable error.
    #[tokio::test]
    async fn test_preflight_g1_dirty_worktree_aborts() {
        let tmp = TempDir::new().unwrap();
        let slug = "g1-dirty-abort";

        // Need a real git binary to exercise G1
        let Some(git) = find_git_binary() else { return };

        // Init a bare main repo
        let main = tmp.path();
        let _ = std::process::Command::new(&git)
            .args(["init", "-q", "-b", "main"])
            .current_dir(main)
            .status();
        std::fs::write(main.join("seed.txt"), "seed\n").unwrap();
        let _ = std::process::Command::new(&git)
            .args(["add", "."])
            .current_dir(main)
            .status();
        let _ = std::process::Command::new(&git)
            .args([
                "-c",
                "user.email=t@t",
                "-c",
                "user.name=t",
                "commit",
                "-q",
                "-m",
                "seed",
            ])
            .current_dir(main)
            .status();

        // Create a worktree on branch cclab/<slug>
        let wt_rel = format!(".aw/worktrees/{}", slug);
        std::fs::create_dir_all(main.join(".aw/worktrees")).unwrap();
        let add_out = std::process::Command::new(&git)
            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
            .current_dir(main)
            .output()
            .unwrap();
        if !add_out.status.success() {
            return; // git worktree add not supported in this test env
        }

        // Set up a valid change inside the worktree
        let wt_root = main.join(&wt_rel);
        let change_dir = wt_root.join(".aw/changes").join(slug);
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::create_dir_all(wt_root.join(".aw/tech-design")).unwrap();

        // config.toml on main (resolve_project_root reads from project_root)
        let config_content = r#"
    [agentic_workflow.repo_platform]
    type = "github"
    repo = "test/repo"
    default_branch = "main"
    auto_commit = false
    auto_pr = false

    [agentic_workflow.tech_design_platform]
    type = "local"
    path = ".aw/tech-design"
    "#;
        std::fs::write(main.join(".aw/config.toml"), config_content).unwrap();

        // Issue backs the change inside the worktree (R4: save() needs it).
        crate::test_util::write_minimal_issue(&wt_root, slug);

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeImplementationReviewed;
        sm.save().unwrap();

        let spec_content = "---\nid: s1\nmain_spec_ref: sdd/logic/s1.md\n---\n\n# S1\n\n";
        std::fs::write(change_dir.join("specs/s1.md"), spec_content).unwrap();

        // DIRTY: write an uncommitted file in the worktree root
        std::fs::write(wt_root.join("dirty.txt"), "uncommitted\n").unwrap();

        let args = json!({
            "project_path": main.to_str().unwrap(),
            "change_id": slug
        });
        let result = execute_workflow(&args, main).await;

        assert!(result.is_err(), "G1 should abort on dirty worktree");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("uncommitted work"),
            "error should mention uncommitted work: {}",
            err_msg
        );
        // Repo state unchanged: change_dir still exists, nothing archived
        assert!(
            change_dir.exists(),
            "change_dir must still exist after G1 abort"
        );
        assert!(
            !wt_root.join(".aw/archive").exists(),
            "no archive should be created when G1 aborts"
        );
    }

    // REQ: change-merge R9 — worktree-first path resolution.
    // Verifies that when a worktree exists at .aw/worktrees/<slug>/,
    // specs promote INSIDE the worktree (not on main), archive lands inside
    // the worktree, and main's .aw/tech-design/ is NOT touched by
    // execute_workflow itself (git merge would bring it later).
    #[tokio::test]
    async fn test_programmatic_merge_uses_worktree_work_root() {
        let tmp = TempDir::new().unwrap();
        let slug = "wt-first-merge";

        // Layout: main + config + tech_design on main; the actual change
        // lives inside .aw/worktrees/<slug>/... (simulated without
        // needing a real git worktree — resolve_worktree_dir just checks
        // that the directory exists).
        let main_root = tmp.path();
        let wt_root = main_root.join(".aw/worktrees").join(slug);
        let change_dir = wt_root.join(".aw/changes").join(slug);
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::create_dir_all(wt_root.join(".aw/tech-design")).unwrap();
        std::fs::create_dir_all(main_root.join(".aw/tech-design")).unwrap();

        // Config lives on main
        let config_content = r#"
    [agentic_workflow.repo_platform]
    type = "github"
    repo = "test/repo"
    default_branch = "main"
    auto_commit = false
    auto_pr = false

    [agentic_workflow.tech_design_platform]
    type = "local"
    path = ".aw/tech-design"
    "#;
        std::fs::write(main_root.join(".aw/config.toml"), config_content).unwrap();

        // Issue backs the change inside the worktree (R4: save() needs it).
        crate::test_util::write_minimal_issue(&wt_root, slug);

        // State lives inside the worktree
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeImplementationReviewed;
        sm.save().unwrap();

        // A minimal spec to promote
        let spec_content =
            "---\nid: wt-spec\nmain_spec_ref: sdd/logic/wt-spec.md\n---\n\n# WT Spec\n\nContent.\n";
        std::fs::write(change_dir.join("specs/wt-spec.md"), spec_content).unwrap();

        // Run with project_root = main_root (the normal CLI case)
        let args = json!({
            "project_path": main_root.to_str().unwrap(),
            "change_id": slug
        });
        let result = execute_workflow(&args, main_root).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Verify: spec landed in worktree's tech_design, NOT on main
        let wt_target = wt_root.join(".aw/tech-design/sdd/logic/wt-spec.md");
        let main_target = main_root.join(".aw/tech-design/sdd/logic/wt-spec.md");
        assert!(
            wt_target.exists(),
            "spec should be promoted inside the worktree at {}",
            wt_target.display()
        );
        assert!(
            !main_target.exists(),
            "spec must NOT be written to main/.aw/tech-design/ — git merge \
             is the only path for main to receive it"
        );

        // Verify: archive landed inside worktree
        assert!(
            !change_dir.exists(),
            "change_dir should be moved to archive"
        );
        let wt_archive_parent = wt_root.join(".aw/archive");
        assert!(
            wt_archive_parent.exists(),
            "archive dir should be inside the worktree"
        );
        let main_archive_parent = main_root.join(".aw/archive");
        assert!(
            !main_archive_parent.exists(),
            "archive must NOT be written to main"
        );
    }
}

#[cfg(test)]
mod section_merge_tests {
    use super::*;

    // ─── Section-Level Merge Unit Tests ──────────────────────────────────

    #[test]
    fn test_parse_markdown_sections_basic() {
        let content = "---\nid: test\nmain_spec_ref: sdd/logic/test.md\n---\n\n# My Spec\n\n## Overview\n\nOverview text.\n\n## Details\n\nDetails text.\n";
        let parsed = parse_markdown_sections(content);
        assert!(parsed.frontmatter.contains("id: test"));
        assert!(parsed.preamble.contains("# My Spec"));
        assert_eq!(parsed.sections.len(), 2);
        assert_eq!(parsed.sections[0].heading, "Overview");
        assert!(parsed.sections[0].body.contains("Overview text."));
        assert_eq!(parsed.sections[1].heading, "Details");
        assert!(parsed.sections[1].body.contains("Details text."));
    }

    #[test]
    fn test_parse_markdown_sections_no_frontmatter() {
        let content = "# Title\n\n## Section A\n\nContent A.\n";
        let parsed = parse_markdown_sections(content);
        assert!(parsed.frontmatter.is_empty());
        assert!(parsed.preamble.contains("# Title"));
        assert_eq!(parsed.sections.len(), 1);
        assert_eq!(parsed.sections[0].heading, "Section A");
    }

    #[test]
    fn test_merge_sections_preserves_untouched() {
        let target = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n## A\n\nA content.\n\n## B\n\nB content.\n\n## C\n\nC content.\n";
        let change = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n## B\n\nB updated.\n";
        let result = merge_sections_into_target(target, change);
        assert!(
            result.contains("A content."),
            "untouched section A must be preserved"
        );
        assert!(
            result.contains("B updated."),
            "changed section B must be updated"
        );
        assert!(
            !result.contains("B content."),
            "old section B must be replaced"
        );
        assert!(
            result.contains("C content."),
            "untouched section C must be preserved"
        );
    }

    #[test]
    fn test_merge_sections_adds_new_section() {
        let target = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n## A\n\nA content.\n";
        let change = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n## A\n\nA content.\n\n## B\n\nNew section B.\n";
        let result = merge_sections_into_target(target, change);
        assert!(
            result.contains("A content."),
            "existing section A preserved"
        );
        assert!(
            result.contains("New section B."),
            "new section B must be added"
        );
    }

    #[test]
    fn test_merge_sections_new_file_creates_all() {
        // When target is empty, all change sections appear
        let target = "";
        let change =
            "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n## A\n\nA.\n\n## B\n\nB.\n";
        let result = merge_sections_into_target(target, change);
        assert!(result.contains("## A"));
        assert!(result.contains("## B"));
    }

    /// Regression test: R2.5 — merge a change-spec that adds one section
    /// to a target file with 5 pre-existing sections. All 6 must be present.
    #[test]
    fn test_merge_sections_five_existing_plus_one_new() {
        let target = "\
    ---\nid: rich\nmain_spec_ref: x/rich.md\n---\n\n# Rich Spec\n\n\
    ## Section One\n\nOne.\n\n\
    ## Section Two\n\nTwo.\n\n\
    ## Section Three\n\nThree.\n\n\
    ## Section Four\n\nFour.\n\n\
    ## Section Five\n\nFive.\n";

        let change = "\
    ---\nid: rich\nmain_spec_ref: x/rich.md\n---\n\n# Rich Spec\n\n\
    ## Section Six\n\nSix added by change.\n";

        let result = merge_sections_into_target(target, change);
        for (i, label) in ["One", "Two", "Three", "Four", "Five"].iter().enumerate() {
            assert!(
                result.contains(&format!("## Section {}", label)),
                "section {} ({}) must be present in merged output",
                i + 1,
                label
            );
        }
        assert!(
            result.contains("## Section Six"),
            "new section Six must be added"
        );
        assert!(
            result.contains("Six added by change."),
            "new section Six body must be present"
        );
    }

    /// Section ordering: target sections that appear between change sections
    /// must be preserved in their original relative order.
    #[test]
    fn test_merge_sections_preserves_order() {
        let target = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n\
    ## A\n\nA.\n\n\
    ## B\n\nB.\n\n\
    ## C\n\nC.\n\n\
    ## D\n\nD.\n";

        // Change modifies A and D, leaving B and C as target-only
        let change = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n\
    ## A\n\nA updated.\n\n\
    ## D\n\nD updated.\n";

        let result = merge_sections_into_target(target, change);

        // All four sections must be present
        assert!(result.contains("A updated."));
        assert!(result.contains("B."));
        assert!(result.contains("C."));
        assert!(result.contains("D updated."));

        // Order must be A, B, C, D
        let a_pos = result.find("## A").unwrap();
        let b_pos = result.find("## B").unwrap();
        let c_pos = result.find("## C").unwrap();
        let d_pos = result.find("## D").unwrap();
        assert!(a_pos < b_pos, "A must come before B");
        assert!(b_pos < c_pos, "B must come before C");
        assert!(c_pos < d_pos, "C must come before D");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_merge.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-trailer>"
    description: "Regression tests for create-change-merge workflow, preflight gates, issue close, and section-level merge helpers."
```
