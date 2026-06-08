---
id: sdd-tools-common-change-impl-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools common change impl tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/common_change_impl.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ImplSubState` | projects/agentic-workflow/src/tools/common_change_impl.rs | enum | pub | 26 |  |
| `MAX_SPEC_REVISIONS` | projects/agentic-workflow/src/tools/common_change_impl.rs | constant | pub | 15 |  |
| `build_spec_execution_order` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 249 | build_spec_execution_order(specs_dir: &Path) -> Vec<String> |
| `build_spec_execution_order_from_paths` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 409 | build_spec_execution_order_from_paths(paths: &[std::path::PathBuf]) -> Vec<String> |
| `collect_all_spec_paths` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 354 | collect_all_spec_paths(change_dir: &Path) -> Vec<std::path::PathBuf> |
| `find_inline_reviews` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 524 | find_inline_reviews(impl_path: &Path) -> (HashSet<String>, HashSet<String>) |
| `is_codegen_eligible_for_spec` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 583 | is_codegen_eligible_for_spec(specs_dir: &Path, spec_id: &str) -> bool |
| `is_codegen_eligible_in_paths` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 433 | is_codegen_eligible_in_paths(paths: &[std::path::PathBuf], spec_id: &str) -> bool |
| `parse_refs_frontmatter` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 458 | parse_refs_frontmatter(path: &Path) -> Vec<String> |
| `resolve_next_impl` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 64 | resolve_next_impl(     change_dir: &Path,     _change_id: &str, ) -> Result<(ImplSubState, Option<String>, Option<String>)> |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateManager;
    use tempfile::TempDir;

    fn write_spec(specs_dir: &Path, spec_id: &str, refs: &[&str]) {
        std::fs::create_dir_all(specs_dir).unwrap();
        let refs_yaml = if refs.is_empty() {
            String::new()
        } else {
            let items = refs
                .iter()
                .map(|r| format!("  - {}", r))
                .collect::<Vec<_>>()
                .join("\n");
            format!("refs:\n{}\n", items)
        };
        std::fs::write(
            specs_dir.join(format!("{}.md", spec_id)),
            format!(
                "---\nid: {}\ntype: spec\n{}---\n# Spec {}\n",
                spec_id, refs_yaml, spec_id
            ),
        )
        .unwrap();
    }

    fn write_impl_md(change_dir: &Path, reviews: &[(&str, &str)]) {
        let mut content = String::from(
            "---\nid: impl\ntype: change_implementation\n---\n# Implementation\n\n## Diff\n\n```diff\n+code\n```\n\n",
        );
        for (spec_id, verdict) in reviews {
            content.push_str(&format!(
                "## Review: {}\n\nverdict: {}\nsummary: test\n\n",
                spec_id, verdict
            ));
        }
        std::fs::write(change_dir.join("implementation.md"), content).unwrap();
    }

    #[test]
    fn test_kahn_ordering_respects_deps() {
        let tmp = TempDir::new().unwrap();
        let specs_dir = tmp.path();
        write_spec(specs_dir, "spec-c", &["spec-a", "spec-b"]);
        write_spec(specs_dir, "spec-a", &[]);
        write_spec(specs_dir, "spec-b", &["spec-a"]);

        let order = build_spec_execution_order(specs_dir);
        assert_eq!(order[0], "spec-a");
        assert_eq!(order[1], "spec-b");
        assert_eq!(order[2], "spec-c");
    }

    #[test]
    fn test_inline_refs_parsed() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("spec-x.md"),
            "---\nid: spec-x\nrefs: [spec-y, spec-z]\n---\n# Spec X\n",
        )
        .unwrap();
        let refs = parse_refs_frontmatter(&tmp.path().join("spec-x.md"));
        assert_eq!(refs, vec!["spec-y", "spec-z"]);
    }

    #[test]
    fn test_find_inline_reviews_approved() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("implementation.md");
        std::fs::write(
            &path,
            "# Impl\n\n## Review: spec-a\n\nverdict: APPROVED\n\n## Review: spec-b\n\nverdict: REVIEWED\n",
        )
        .unwrap();
        let (reviewed, approved) = find_inline_reviews(&path);
        assert!(reviewed.contains("spec-a"));
        assert!(reviewed.contains("spec-b"));
        assert!(approved.contains("spec-a"));
        assert!(!approved.contains("spec-b"));
    }

    #[test]
    fn test_resolve_no_specs() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_created")
                    .unwrap();
            sm.save().unwrap();
        }
        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert_eq!(sub_state, ImplSubState::NoSpecs);
    }

    #[test]
    fn test_resolve_first_spec() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_created")
                    .unwrap();
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);

        let (sub_state, new_id, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert!(
            matches!(sub_state, ImplSubState::ImplementSpecCode { ref spec_id, is_first: true } if spec_id == "spec-a")
        );
        assert_eq!(new_id, Some("spec-a".to_string()));
    }

    #[test]
    fn test_resolve_write_diff() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_created")
                    .unwrap();
            sm.state_mut().current_task_id = Some("spec-a".into());
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);

        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert_eq!(sub_state, ImplSubState::WriteDiff);
    }

    #[test]
    fn test_resolve_review_after_impl_written() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_created")
                    .unwrap();
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);
        write_impl_md(&change_dir, &[]);

        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert!(
            matches!(sub_state, ImplSubState::ReviewSpec { ref spec_id } if spec_id == "spec-a")
        );
    }

    #[test]
    fn test_resolve_all_approved_triggers_merge() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_reviewed")
                    .unwrap();
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);
        write_impl_md(&change_dir, &[("spec-a", "APPROVED")]);

        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert_eq!(sub_state, ImplSubState::AdvanceToMerge);
    }

    #[test]
    fn test_resolve_terminal_failure() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_reviewed")
                    .unwrap();
            sm.state_mut().task_revisions.insert("spec-a".into(), 2);
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);
        write_impl_md(&change_dir, &[("spec-a", "REJECTED")]);

        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert!(
            matches!(sub_state, ImplSubState::TerminalFailure { ref spec_id, revisions: 2 } if spec_id == "spec-a")
        );
    }

    #[test]
    fn test_impl_spec_phase_tracking_in_state() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");

        // Set impl_spec_phase["spec-a"] = "code"
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_created")
                    .unwrap();
            sm.state_mut().current_task_id = Some("spec-a".into());
            sm.state_mut()
                .impl_spec_phase
                .insert("spec-a".into(), "code".into());
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);

        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert!(
            matches!(sub_state, ImplSubState::BuildCheck { ref spec_id } if spec_id == "spec-a")
        );

        // Now set to "tests"
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut()
                .impl_spec_phase
                .insert("spec-a".into(), "tests".into());
            sm.save().unwrap();
        }
        let (sub_state2, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert!(
            matches!(sub_state2, ImplSubState::TestCountCheck { ref spec_id } if spec_id == "spec-a")
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/common_change_impl.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
      - "<module-trailer>"
    description: "Common change implementation helper regression tests."
```
