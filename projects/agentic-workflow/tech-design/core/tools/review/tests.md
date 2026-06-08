---
id: sdd-tools-review-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/review.rs | function | pub | 40 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/review.rs | function | pub | 135 | execute(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change(change_id: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();
        let change_dir = project_root.join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), change_id);
        (temp_dir, project_root)
    }

    // -- Inline review tests --

    #[test]
    fn test_review_context_approved() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        std::fs::write(
            change_dir.join("spec_context.md"),
            "---\nid: test-change\ntype: spec_context\n---\n\n# Spec Context\n\nContent.\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "spec_context",
            "verdict": "PASS",
            "summary": "All checklist items satisfied",
            "checklist_results": [
                {"item": "All spec groups scanned", "passed": true},
                {"item": "Each spec has relevance score", "passed": true}
            ]
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("spec_context.md"));
        assert!(result.contains("APPROVED")); // PASS normalized

        let content = std::fs::read_to_string(change_dir.join("spec_context.md")).unwrap();
        // APPROVED: no review fields in frontmatter, no # Reviews section
        assert!(!content.contains("review_verdict:"));
        assert!(!content.contains("# Reviews"));
        // Original content preserved
        assert!(content.contains("# Spec Context"));
        assert!(content.contains("Content."));
    }

    #[test]
    fn test_review_gap_reviewed() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        std::fs::write(
            change_dir.join("gap_codebase_spec.md"),
            "---\nid: test-change\ntype: gap\n---\n\n# Gap Analysis\n\nGap content.\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "gap_codebase_spec",
            "verdict": "REVIEWED",
            "summary": "Missing file paths for two gaps",
            "issues": [
                {"severity": "HIGH", "description": "Gap #3 missing file path", "recommendation": "Add crate path"},
                {"severity": "MEDIUM", "description": "Severity not assigned for gap #5"}
            ]
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("gap_codebase_spec.md"));
        assert!(result.contains("REVIEWED"));
        assert!(result.contains("1 high"));
        assert!(result.contains("1 medium"));

        let content = std::fs::read_to_string(change_dir.join("gap_codebase_spec.md")).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("review_iteration: 1"));
        assert!(content.contains("# Reviews"));
        assert!(content.contains("**Verdict**: REVIEWED"));
        assert!(content.contains("**[HIGH]** Gap #3 missing file path"));
        assert!(content.contains("# Gap Analysis")); // original preserved
    }

    #[test]
    fn test_review_spec_requires_spec_id() {
        let (_tmp, root) = setup_change("test-change");
        let args = json!({
            "change_id": "test-change",
            "file": "spec",
            "verdict": "PASS",
            "summary": "Spec looks good overall"
        });
        let result = execute(&args, &root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("spec_id is required"));
    }

    #[test]
    fn test_review_spec_with_spec_id() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("per-task-loop.md"),
            "---\nid: per-task-loop\ntype: spec\n---\n\n# Spec\n\nContent.\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "spec",
            "spec_id": "per-task-loop",
            "verdict": "PASS",
            "summary": "Spec requirements and scenarios are complete"
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("per-task-loop.md"));
    }

    #[test]
    fn test_review_invalid_file() {
        let (_tmp, root) = setup_change("test-change");
        let args = json!({
            "change_id": "test-change",
            "file": "invalid_type",
            "verdict": "PASS",
            "summary": "This should fail"
        });
        let result = execute(&args, &root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid file"));
    }

    #[test]
    fn test_review_proposal_with_iteration() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        std::fs::write(
            change_dir.join("proposal.md"),
            "---\nid: test-change\ntype: proposal\n---\n\n# Proposal\n\nContent.\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "proposal",
            "verdict": "REVIEWED",
            "summary": "Proposal needs minor adjustments",
            "iteration": 2
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("proposal.md"));

        let content = std::fs::read_to_string(change_dir.join("proposal.md")).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("review_iteration: 2"));
        assert!(content.contains("Iteration 2"));
    }

    #[test]
    fn test_review_spec_frontmatter_has_review_fields() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("auth-flow.md"),
            "---\nid: auth-flow\ntype: spec\n---\n\n# Auth Flow\n\nContent.\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "spec",
            "spec_id": "auth-flow",
            "verdict": "REVIEWED",
            "summary": "Missing error scenarios",
            "issues": [{"severity": "MEDIUM", "description": "No error case for expired token"}]
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("auth-flow.md"));

        let content = std::fs::read_to_string(specs_dir.join("auth-flow.md")).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("review_iteration: 1"));
        assert!(content.contains("# Auth Flow")); // original preserved
    }

    #[test]
    fn test_review_approved_clears_previous_review() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        // Artifact with existing inline review
        std::fs::write(
            change_dir.join("proposal.md"),
            "---\nid: test-change\ntype: proposal\nreview_verdict: REVIEWED\nreview_iteration: 1\n---\n\n# Proposal\n\nContent.\n\n# Reviews\n\n## Review (Iteration 1)\n\nOld review.\n",
        ).unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "proposal",
            "verdict": "APPROVED",
            "summary": "All good now",
            "iteration": 2
        });
        execute(&args, &root).unwrap();

        let content = std::fs::read_to_string(change_dir.join("proposal.md")).unwrap();
        assert!(!content.contains("review_verdict"));
        assert!(!content.contains("review_iteration"));
        assert!(!content.contains("# Reviews"));
        assert!(content.contains("# Proposal"));
        assert!(content.contains("Content."));
    }

    #[test]
    fn test_review_updates_previous_review() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        // Artifact with existing inline review
        std::fs::write(
            change_dir.join("proposal.md"),
            "---\nid: test-change\ntype: proposal\nreview_verdict: REVIEWED\nreview_iteration: 1\n---\n\n# Proposal\n\nContent.\n\n# Reviews\n\n## Review (Iteration 1)\n\nOld review.\n",
        ).unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "proposal",
            "verdict": "REVIEWED",
            "summary": "Still needs work",
            "iteration": 2
        });
        execute(&args, &root).unwrap();

        let content = std::fs::read_to_string(change_dir.join("proposal.md")).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("review_iteration: 2"));
        assert!(content.contains("# Reviews"));
        assert!(content.contains("Iteration 2"));
        assert!(!content.contains("Old review.")); // old review stripped
        assert!(content.contains("Still needs work"));
    }

    // -- Inline implementation review tests --

    #[test]
    fn test_review_implementation_global_inline() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        // Create impl.md (auto-generated by implementation workflow)
        std::fs::write(
            change_dir.join("impl.md"),
            "# Implementation Diff\n\n```diff\n+fn new_fn() {}\n```\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "implementation",
            "verdict": "PASS",
            "summary": "Implementation looks good"
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("impl.md"));
        assert!(result.contains("APPROVED"));

        let content = std::fs::read_to_string(change_dir.join("impl.md")).unwrap();
        // APPROVED: no review fields, no # Reviews section
        assert!(!content.contains("review_verdict"));
        assert!(!content.contains("# Reviews"));
        // Original diff preserved
        assert!(content.contains("# Implementation Diff"));
        assert!(content.contains("+fn new_fn()"));
    }

    #[test]
    fn test_review_implementation_task_inline() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        std::fs::write(
            change_dir.join("impl_2.1.md"),
            "# Implementation Diff — Task 2.1\n\n```diff\n+fn task_fn() {}\n```\n",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "implementation",
            "task_id": "2.1",
            "verdict": "REVIEWED",
            "summary": "Task 2.1 needs fixes",
            "issues": [{"severity": "HIGH", "description": "Missing error handling"}]
        });
        let result = execute(&args, &root).unwrap();
        assert!(result.contains("impl_2.1.md"));
        assert!(result.contains("REVIEWED"));

        let content = std::fs::read_to_string(change_dir.join("impl_2.1.md")).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("review_iteration: 1"));
        assert!(content.contains("# Reviews"));
        assert!(content.contains("**[HIGH]** Missing error handling"));
        // Original diff preserved
        assert!(content.contains("# Implementation Diff"));
        assert!(content.contains("+fn task_fn()"));
    }

    #[test]
    fn test_review_implementation_independent_tasks_inline() {
        let (_tmp, root) = setup_change("test-change");
        let change_dir = root.join(".aw/changes/test-change");
        std::fs::write(
            change_dir.join("impl_1.1.md"),
            "# Implementation Diff — Task 1.1\n\n```diff\n+fn a() {}\n```\n",
        )
        .unwrap();
        std::fs::write(
            change_dir.join("impl_1.2.md"),
            "# Implementation Diff — Task 1.2\n\n```diff\n+fn b() {}\n```\n",
        )
        .unwrap();

        // Review task 1.1 — APPROVED
        execute(
            &json!({
                "change_id": "test-change",
                "file": "implementation",
                "task_id": "1.1",
                "verdict": "PASS",
                "summary": "Task 1.1 looks good"
            }),
            &root,
        )
        .unwrap();

        // Review task 1.2 — REVIEWED
        execute(
            &json!({
                "change_id": "test-change",
                "file": "implementation",
                "task_id": "1.2",
                "verdict": "REVIEWED",
                "summary": "Task 1.2 needs fixes",
                "issues": [{"severity": "MEDIUM", "description": "Missing test"}]
            }),
            &root,
        )
        .unwrap();

        // Task 1.1: APPROVED → no review section
        let c1 = std::fs::read_to_string(change_dir.join("impl_1.1.md")).unwrap();
        assert!(!c1.contains("# Reviews"));
        assert!(c1.contains("+fn a()"));

        // Task 1.2: REVIEWED → has review section
        let c2 = std::fs::read_to_string(change_dir.join("impl_1.2.md")).unwrap();
        assert!(c2.contains("# Reviews"));
        assert!(c2.contains("review_verdict: REVIEWED"));
        assert!(c2.contains("+fn b()"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/review.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
      - "<module-trailer>"
    description: "Regression tests for review file writing and update behavior."
```
