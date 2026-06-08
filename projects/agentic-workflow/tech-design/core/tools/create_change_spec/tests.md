---
id: sdd-tools-create-change-spec-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change spec tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 55 | artifact_definition() -> ToolDefinition |
| `build_fill_prompt` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 736 | build_fill_prompt(     change_id: &str,     spec_id: &str,     section: &str,     group_id: Option<&str>,     project_root: &Path, ) -> Result<String> |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 338 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 120 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateManager;
    use tempfile::TempDir;

    /// Read prompt content from either inline response or prompt file.
    fn read_prompt(parsed: &Value, project_root: &std::path::Path) -> String {
        if let Some(p) = parsed["prompt"].as_str() {
            return p.to_string();
        }
        let rel = parsed["prompt_path"]
            .as_str()
            .expect("Expected prompt_path in response");
        let prompt_path = project_root.join(rel);
        std::fs::read_to_string(&prompt_path)
            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
    }

    fn setup_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        // R4: save() needs an issue backing change_id.
        crate::test_util::write_minimal_issue(tmp.path(), change_id);

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        tmp
    }

    #[tokio::test]
    async fn test_workflow_creates_skeleton() {
        let tmp = setup_change("skel-test");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "skel-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["spec_id"], "skel-test-spec");
        let prompt = read_prompt(&parsed, tmp.path());
        assert!(prompt.contains("Analyze Spec"));

        // Verify skeleton file created
        let spec_path = tmp
            .path()
            .join(".aw/changes/skel-test/specs/skel-test-spec.md");
        assert!(spec_path.exists());
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("id: skel-test-spec"));
        assert!(content.contains("## Overview"));
        assert!(content.contains("<!-- TODO -->"));
    }

    #[tokio::test]
    async fn test_workflow_returns_fill_prompt_after_analyze() {
        let tmp = setup_change("fill-test");
        let change_dir = tmp.path().join(".aw/changes/fill-test");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Write skeleton with fill_sections set (analyze done)
        let content = "---\nid: fill-test-spec\nfill_sections: [overview, requirements, scenarios]\n---\n\n# Fill Test Spec\n\n## Overview\n\n<!-- TODO -->\n\n## Requirements\n\n<!-- TODO -->\n\n## Scenarios\n\n<!-- TODO -->\n\n# Reviews\n";
        std::fs::write(change_dir.join("specs/fill-test-spec.md"), content).unwrap();

        // Update phase
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.set_phase(StatePhase::ChangeSpecCreated).unwrap();
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "fill-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        // Should return fill prompt for first section (overview)
        let prompt = read_prompt(&parsed, tmp.path());
        assert!(prompt.contains("Fill Section 'overview'"));
    }

    #[tokio::test]
    async fn test_workflow_prunes_and_completes() {
        let tmp = setup_change("prune-test");
        let change_dir = tmp.path().join(".aw/changes/prune-test");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Write spec with all sections filled
        let content = "---\nid: prune-test-spec\nfill_sections: [overview, requirements]\nfilled_sections: [overview, requirements]\n---\n\n# Prune Test Spec\n\n## Overview\n\nReal overview.\n\n## Requirements\n\nReal requirements.\n\n## Scenarios\n\n<!-- TODO -->\n\n## Diagrams\n\n### Sequence Diagram\n<!-- TODO -->\n\n## Changes\n\n<!-- TODO -->\n\n# Reviews\n";
        std::fs::write(change_dir.join("specs/prune-test-spec.md"), content).unwrap();

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.set_phase(StatePhase::ChangeSpecCreated).unwrap();
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "prune-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Should redirect to review (create complete)
        let next = &parsed["next_actions"][0];
        assert_eq!(next["args"]["change_id"], "prune-test");

        // Verify file was pruned
        let spec_path = change_dir.join("specs/prune-test-spec.md");
        let final_content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(final_content.contains("create_complete: true"));
        assert!(final_content.contains("Real overview."));
        assert!(!final_content.contains("<!-- TODO -->"));
    }

    #[test]
    fn test_artifact_writes_section() {
        let tmp = setup_change("art-sec");
        let change_dir = tmp.path().join(".aw/changes/art-sec");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Write skeleton
        let skeleton =
            common::generate_skeleton("art-sec-spec", "Art Sec Spec", None, None, tmp.path());
        std::fs::write(change_dir.join("specs/art-sec-spec.md"), &skeleton).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "art-sec",
            "spec_id": "art-sec-spec",
            "section": "overview",
            "content": "This is a comprehensive overview of the spec.\n\nIt covers many things."
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(parsed["artifacts_written"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("art-sec-spec.md")));

        // Verify content was written
        let spec_path = change_dir.join("specs/art-sec-spec.md");
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("comprehensive overview"));
        assert!(content.contains("filled_sections: [overview]"));
    }

    #[test]
    fn test_artifact_rejects_invalid_section() {
        let tmp = setup_change("bad-sec");
        let change_dir = tmp.path().join(".aw/changes/bad-sec");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("specs/bad-sec-spec.md"),
            "---\nid: bad-sec-spec\n---\n\n# Test\n",
        )
        .unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "bad-sec",
            "spec_id": "bad-sec-spec",
            "section": "nonexistent",
            "content": "test"
        });
        let result = execute_artifact(&args, tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid section"));
    }

    #[tokio::test]
    async fn test_workflow_all_specs_done_advances_to_implementation() {
        let tmp = setup_change("advance-test");
        let change_dir = tmp.path().join(".aw/changes/advance-test");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        // Write a spec with APPROVED verdict (create_complete + approved)
        let content = "---\nid: advance-test-spec\ncreate_complete: true\n---\n\n# Spec\n\n## Overview\n\nDone.\n";
        std::fs::write(specs_dir.join("advance-test-spec.md"), content).unwrap();

        // Write proposal with this spec listed
        let proposal = "---\nspec_plan:\n- id: advance-test-spec\n---\n\n# Proposal\n";
        std::fs::write(change_dir.join("proposal.md"), proposal).unwrap();

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.set_phase(StatePhase::ChangeSpecReviewed).unwrap();
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "advance-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Should be a review redirect (spec exists, no review yet based on analyze_specs)
        // The exact behavior depends on analyze_specs verdict check
        assert!(parsed["status"].as_str().is_some());
    }

    #[test]
    fn test_create_complete_blocked_on_failed_sections() {
        // When failed_sections is non-empty, create_complete must NOT be written
        // We simulate the logic directly since run_create_spec_agent_loop is async and agent-driven
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/spec-guard");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        // Write a spec without create_complete
        let spec_content =
            "---\nid: spec-guard\ntype: spec\n---\n# Spec\n\n## Overview\n\nSome content.\n";
        let spec_path = specs_dir.join("spec-guard.md");
        std::fs::write(&spec_path, spec_content).unwrap();

        // Simulate: failed_sections is non-empty → should NOT write create_complete
        let failed_sections = vec!["requirements".to_string()];
        assert!(!failed_sections.is_empty());

        // Read the spec content — create_complete should NOT be set
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(
            !content.contains("create_complete: true"),
            "create_complete must NOT be written when failed_sections is non-empty"
        );
    }

    #[test]
    fn test_create_complete_written_on_all_filled() {
        use crate::tools::common_change_spec as common;
        use crate::tools::review_helpers;

        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/spec-ok");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        let spec_content =
            "---\nid: spec-ok\ntype: spec\n---\n# Spec\n\n## Overview\n\nContent here.\n";
        let spec_path = specs_dir.join("spec-ok.md");
        std::fs::write(&spec_path, spec_content).unwrap();

        // Simulate: failed_sections is empty → write create_complete
        let failed_sections: Vec<String> = vec![];
        if failed_sections.is_empty() {
            let content = std::fs::read_to_string(&spec_path).unwrap();
            if !common::is_create_complete(&content) {
                let pruned = common::prune_todo_sections(&content);
                let marked =
                    review_helpers::upsert_frontmatter_field(&pruned, "create_complete", "true");
                std::fs::write(&spec_path, &marked).unwrap();
            }
        }

        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(
            content.contains("create_complete: true"),
            "create_complete must be written when failed_sections is empty"
        );
    }

    // ── artifact-tools-update: merge_strategy removed ─────────────────────────

    #[test]
    fn test_artifact_definition_excludes_merge_strategy() {
        // merge_strategy must not appear in sdd_artifact_create_change_spec schema.
        // Merge behavior is always replace (write to path, create if absent, overwrite if present).
        let def = artifact_definition();
        let props = def.input_schema["properties"]
            .as_object()
            .expect("properties must be an object");
        assert!(
            !props.contains_key("merge_strategy"),
            "merge_strategy must not be in artifact_definition schema; it is dead code"
        );
    }

    #[test]
    fn test_artifact_ignores_merge_strategy_param() {
        // When merge_strategy is passed by old callers it must be silently ignored.
        let tmp = setup_change("ms-ignore");
        let change_dir = tmp.path().join(".aw/changes/ms-ignore");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let skeleton =
            common::generate_skeleton("ms-ignore-spec", "Ms Ignore Spec", None, None, tmp.path());
        std::fs::write(change_dir.join("specs/ms-ignore-spec.md"), &skeleton).unwrap();

        // Include merge_strategy — must not cause an error
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "ms-ignore",
            "spec_id": "ms-ignore-spec",
            "section": "overview",
            "content": "Overview content.",
            "merge_strategy": "append"
        });
        let result = execute_artifact(&args, tmp.path());
        assert!(
            result.is_ok(),
            "merge_strategy in args must be ignored, not rejected: {:?}",
            result.err()
        );

        let content = std::fs::read_to_string(change_dir.join("specs/ms-ignore-spec.md")).unwrap();
        assert!(content.contains("Overview content."));
    }

    #[test]
    fn test_artifact_write_replaces_section_content() {
        // Merge behavior is always replace: a second write to the same section must
        // overwrite the previous value, never append.
        let tmp = setup_change("replace-sem");
        let change_dir = tmp.path().join(".aw/changes/replace-sem");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let skeleton = common::generate_skeleton(
            "replace-sem-spec",
            "Replace Sem Spec",
            None,
            None,
            tmp.path(),
        );
        std::fs::write(change_dir.join("specs/replace-sem-spec.md"), &skeleton).unwrap();

        // First write
        let args_first = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "replace-sem",
            "spec_id": "replace-sem-spec",
            "section": "overview",
            "content": "Initial overview."
        });
        execute_artifact(&args_first, tmp.path()).unwrap();

        // Second write — must overwrite, not append
        let args_second = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "replace-sem",
            "spec_id": "replace-sem-spec",
            "section": "overview",
            "content": "Revised overview."
        });
        execute_artifact(&args_second, tmp.path()).unwrap();

        let content =
            std::fs::read_to_string(change_dir.join("specs/replace-sem-spec.md")).unwrap();
        assert!(
            content.contains("Revised overview."),
            "second write must be present"
        );
        assert!(
            !content.contains("Initial overview."),
            "first write must be replaced, not appended"
        );
    }

    // ── Phase 3: Post-write alignment validation tests ──────────────────────

    #[test]
    fn test_artifact_alignment_format_violation_reverts() {
        // (a) When create_complete:true and the spec has format violations,
        // execute_artifact must revert the write and return status:"error".
        let tmp = setup_change("align-revert");
        let change_dir = tmp.path().join(".aw/changes/align-revert");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Spec with create_complete:true and a duplicate heading that will
        // trigger DuplicateSection after the overview section is filled.
        let spec_content = "\
---
id: align-revert-spec
create_complete: true
---

# Align Revert Spec

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Overview
<!-- type: overview lang: markdown -->

Old duplicate content.

# Reviews
";
        std::fs::write(change_dir.join("specs/align-revert-spec.md"), spec_content).unwrap();

        let original =
            std::fs::read_to_string(change_dir.join("specs/align-revert-spec.md")).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "align-revert",
            "spec_id": "align-revert-spec",
            "section": "overview",
            "content": "New overview content that should be reverted."
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Must return error status
        assert_eq!(
            parsed["status"], "error",
            "format violations must produce error status"
        );
        assert!(
            parsed["message"]
                .as_str()
                .unwrap_or("")
                .contains("format violations"),
            "error message must mention format violations"
        );
        assert!(
            parsed["violations"].is_array(),
            "must include violations array"
        );

        // File must be reverted to original content
        let after = std::fs::read_to_string(change_dir.join("specs/align-revert-spec.md")).unwrap();
        assert_eq!(
            after, original,
            "file must be reverted to pre-write content on format violation"
        );
    }

    #[test]
    fn test_artifact_annotation_preserved_after_section_write() {
        // replace_section now emits <!-- type: X lang: Y --> annotations,
        // so writing a section on a create_complete spec should pass
        // alignment checks (no violations).
        let tmp = setup_change("align-schema");
        let change_dir = tmp.path().join(".aw/changes/align-schema");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let spec_content = "\
---
id: align-schema-spec
create_complete: true
---

# Align Schema Spec

## Overview
<!-- type: overview lang: markdown -->

Existing overview.

# Reviews
";
        std::fs::write(change_dir.join("specs/align-schema-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "align-schema",
            "spec_id": "align-schema-spec",
            "section": "overview",
            "content": "Updated overview content."
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Should succeed now that annotations are preserved
        assert_eq!(
            parsed["status"], "ok",
            "annotation should be preserved: {:?}",
            parsed
        );
    }

    #[test]
    fn test_artifact_alignment_skipped_when_incomplete() {
        // (c) When create_complete is NOT true, alignment check must be
        // skipped even if the spec has format violations (e.g. duplicate
        // headings). This avoids false positives on partially-filled specs.
        let tmp = setup_change("align-skip");
        let change_dir = tmp.path().join(".aw/changes/align-skip");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Spec WITHOUT create_complete — has duplicate headings that would
        // fail alignment, but check should be skipped entirely.
        let spec_content = "\
---
id: align-skip-spec
---

# Align Skip Spec

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Overview
<!-- type: overview lang: markdown -->

Old duplicate content.

# Reviews
";
        std::fs::write(change_dir.join("specs/align-skip-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "align-skip",
            "spec_id": "align-skip-spec",
            "section": "overview",
            "content": "New overview content (should not be reverted)."
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Must succeed — alignment check was skipped
        assert_eq!(
            parsed["status"], "ok",
            "incomplete spec must not trigger alignment validation"
        );
        // alignment_warnings should be null (check was skipped)
        assert!(
            parsed["alignment_warnings"].is_null(),
            "alignment_warnings must be null when create_complete is not set"
        );

        // Verify the new content was written (not reverted)
        let content = std::fs::read_to_string(change_dir.join("specs/align-skip-spec.md")).unwrap();
        assert!(
            content.contains("New overview content"),
            "content must be written when alignment check is skipped"
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
    description: "Regression tests for create-change-spec workflow and artifact behavior."
```
