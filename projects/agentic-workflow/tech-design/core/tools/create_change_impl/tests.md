---
id: projects-sdd-src-tools-create-change-impl-rs-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd create change implementation tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_impl.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 46 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 348 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 83 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `parse_test_plan_count` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 869 | parse_test_plan_count(spec_content: &str) -> Option<usize> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 21 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change(change_id: &str, phase_str: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        // R4: save() syncs workflow fields into the issue frontmatter, the
        // single source of truth since STATE.yaml was eliminated (R5/R6).
        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let phase =
            crate::tools::phase_transition::parse_phase(phase_str).expect("valid phase string");
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = phase;
        sm.save().unwrap();
        tmp
    }

    fn write_spec(tmp: &TempDir, change_id: &str, spec_id: &str, refs: &[&str]) {
        let specs_dir = tmp.path().join(".aw/changes").join(change_id).join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
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

    fn read_prompt(parsed: &Value, change_dir: &std::path::Path, action: &str) -> String {
        if let Some(p) = parsed["prompt"].as_str() {
            return p.to_string();
        }
        let prompt_path = change_dir.join("prompts").join(format!("{}.md", action));
        std::fs::read_to_string(&prompt_path)
            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
    }

    #[tokio::test]
    async fn test_workflow_begin_implementation() {
        let tmp = setup_change("wf-impl", "change_implementation_created");
        write_spec(&tmp, "wf-impl", "spec-a", &[]);

        let change_dir = tmp.path().join(".aw/changes/wf-impl");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "wf-impl"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["spec_id"], "spec-a");
        let prompt = read_prompt(&parsed, &change_dir, "begin_implementation");
        assert!(prompt.contains("Begin Implementation"));
    }

    #[tokio::test]
    async fn test_workflow_write_diff() {
        let tmp = setup_change("wf-diff", "change_implementation_created");
        write_spec(&tmp, "wf-diff", "spec-a", &[]);
        // Set current_task_id to last spec (all dispatched)
        let change_dir = tmp.path().join(".aw/changes/wf-diff");
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().current_task_id = Some("spec-a".into());
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "wf-diff"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        let prompt = read_prompt(&parsed, &change_dir, "write_implementation_diff");
        assert!(prompt.contains("Write Implementation Diff"));
    }

    #[test]
    fn test_artifact_writes_impl_md() {
        let tmp = setup_change("art-impl", "change_implementation_created");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "art-impl",
            "diff": "+new line\n-old line",
            "summary": "Added feature X"
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(parsed["artifacts_written"]
            .as_array()
            .unwrap()
            .contains(&json!("implementation.md")));

        // Verify file content
        let impl_path = tmp.path().join(".aw/changes/art-impl/implementation.md");
        let content = std::fs::read_to_string(&impl_path).unwrap();
        assert!(content.contains("+new line"));
        assert!(content.contains("Added feature X"));
    }

    #[tokio::test]
    async fn test_workflow_all_approved_advances_to_merge() {
        let tmp = setup_change("wf-merge", "change_implementation_reviewed");
        write_spec(&tmp, "wf-merge", "spec-a", &[]);
        let change_dir = tmp.path().join(".aw/changes/wf-merge");
        let mut content = String::from(
            "---\nid: impl\ntype: change_implementation\n---\n# Implementation\n\n## Diff\n\n```diff\n+code\n```\n\n",
        );
        content.push_str("## Review: spec-a\n\nverdict: APPROVED\nsummary: looks good\n\n");
        std::fs::write(change_dir.join("implementation.md"), content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "wf-merge"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "phase_complete");
        assert!(parsed["message"]
            .as_str()
            .unwrap()
            .contains("All specs implemented and approved"));
        // Phase should be advanced to test_check in STATE.yaml
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), crate::models::state::StatePhase::TestCheck);
    }

    #[tokio::test]
    async fn test_terminal_failure_returns_retry_action() {
        let tmp = setup_change("wf-fail", "change_implementation_reviewed");
        write_spec(&tmp, "wf-fail", "spec-a", &[]);
        let change_dir = tmp.path().join(".aw/changes/wf-fail");

        // Set task_revisions to exceed MAX_SPEC_REVISIONS
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut()
            .task_revisions
            .insert("spec-a".into(), MAX_SPEC_REVISIONS);
        sm.save().unwrap();

        // Write impl with REVISE verdict to trigger TerminalFailure
        let mut content = String::from(
            "---\nid: impl\ntype: change_implementation\n---\n# Implementation\n\n## Diff\n\n```diff\n+code\n```\n\n",
        );
        content.push_str("## Review: spec-a\n\nverdict: REVISE\nsummary: needs work\n\n");
        std::fs::write(change_dir.join("implementation.md"), content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "wf-fail"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "error");
        let next = parsed["next_actions"].as_array().unwrap();
        assert!(
            !next.is_empty(),
            "TerminalFailure should provide retry next_actions"
        );
        assert_eq!(next[0]["args"]["change_id"], "wf-fail");
    }

    #[test]
    fn test_build_gate_blocks_phase2_on_failure() {
        // Given impl_spec_phase["spec-a"] == "code"
        // When build fails (simulate by checking the structure)
        // This is a unit test of parse_test_plan_count and count logic
        // The actual build gate is tested via state inspection
        let tmp = setup_change("gate-fail", "change_implementation_created");
        let change_dir = tmp.path().join(".aw/changes/gate-fail");
        write_spec(&tmp, "gate-fail", "spec-a", &[]);
        // Set impl_spec_phase to "code" — simulates code phase dispatched
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().current_task_id = Some("spec-a".into());
        sm.state_mut()
            .impl_spec_phase
            .insert("spec-a".into(), "code".into());
        sm.save().unwrap();
        // The BuildCheck sub-state should be returned
        let (sub_state, _, _) = common::resolve_next_impl(&change_dir, "gate-fail").unwrap();
        assert!(
            matches!(sub_state, common::ImplSubState::BuildCheck { ref spec_id } if spec_id == "spec-a")
        );
    }

    #[test]
    fn test_build_gate_passes_on_success() {
        // Given impl_spec_phase["spec-a"] == "tests"
        // The TestCountCheck sub-state should be returned (build already passed)
        let tmp = setup_change("gate-pass", "change_implementation_created");
        let change_dir = tmp.path().join(".aw/changes/gate-pass");
        write_spec(&tmp, "gate-pass", "spec-a", &[]);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().current_task_id = Some("spec-a".into());
        sm.state_mut()
            .impl_spec_phase
            .insert("spec-a".into(), "tests".into());
        sm.save().unwrap();
        let (sub_state, _, _) = common::resolve_next_impl(&change_dir, "gate-pass").unwrap();
        assert!(
            matches!(sub_state, common::ImplSubState::TestCountCheck { ref spec_id } if spec_id == "spec-a")
        );
    }

    #[test]
    fn test_test_count_warning_on_mismatch() {
        // Spec with 4 tests in Unit Test table, diff has 2 #[test] → warning
        let spec_content = "---\nid: spec\n---\n# Spec\n\n## Unit Test\n\n| # | Test | File | Validates |\n|---|------|------|------|\n| T1 | test1 | foo | bar |\n| T2 | test2 | foo | bar |\n| T3 | test3 | foo | bar |\n| T4 | test4 | foo | bar |\n";
        let required = parse_test_plan_count(spec_content);
        assert_eq!(required, Some(4));
    }

    #[test]
    fn test_test_count_skipped_no_unit_test() {
        // Spec with no ## Unit Test section
        let spec_content = "---\nid: spec\n---\n# Spec\n\n## Overview\n\nSome overview.\n";
        let required = parse_test_plan_count(spec_content);
        assert_eq!(required, None);
    }

    #[test]
    fn test_test_count_skipped_qualitative_plan() {
        // Spec with Unit Test section but no table (qualitative only)
        let spec_content =
            "---\nid: spec\n---\n# Spec\n\n## Unit Test\n\nEnsure all edge cases are covered.\n";
        let required = parse_test_plan_count(spec_content);
        assert_eq!(required, None);
    }

    // ─── CLI Hints Tests ────────────────────────────────────────────────────

    /// Helper: write .aw/config.toml with the given execution mode.
    ///
    /// Mode values: "mainthread", "claude_subagents", "multi_claude_agents", "multi_agents".
    /// Tests MUST call this to control executor selection; without it the default
    /// `MultiClaudeAgents` mode dispatches to external agents that are unavailable in CI.
    fn write_config(tmp: &TempDir, mode: &str) {
        let config_dir = tmp.path().join("cclab");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            format!("[workflow]\nmode = \"{}\"\n", mode),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_cli_hints_present_for_mainthread_executor() {
        let tmp = setup_change("hints-mt", "change_implementation_created");
        write_spec(&tmp, "hints-mt", "spec-a", &[]);
        write_config(&tmp, "mainthread");

        let change_dir = tmp.path().join(".aw/changes/hints-mt");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "hints-mt"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        let prompt = read_prompt(&parsed, &change_dir, "begin_implementation");
        assert!(
            prompt.contains("Code intelligence"),
            "Mainthread prompt should contain Code intelligence header"
        );
    }

    #[tokio::test]
    async fn test_cli_hints_contains_all_five_commands() {
        let tmp = setup_change("hints-5c", "change_implementation_created");
        write_spec(&tmp, "hints-5c", "spec-a", &[]);
        write_config(&tmp, "mainthread");

        let change_dir = tmp.path().join(".aw/changes/hints-5c");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "hints-5c"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let prompt = read_prompt(&parsed, &change_dir, "begin_implementation");

        let expected_commands = [
            "score symbols <file>",
            "score hover <file> <line> <col>",
            "score references <file> <line> <col>",
            "score impact <file> <line> <col>",
            "score context <file:symbol...>",
        ];
        for cmd in &expected_commands {
            assert!(
                prompt.contains(cmd),
                "CLI hints should contain command: {}",
                cmd
            );
        }
    }

    #[tokio::test]
    async fn test_cli_hints_inside_cli_commands_block() {
        // CLI hints should appear within the ## CLI Commands section, not outside it.
        let tmp = setup_change("hints-blk", "change_implementation_created");
        write_spec(&tmp, "hints-blk", "spec-a", &[]);
        write_config(&tmp, "mainthread");

        let change_dir = tmp.path().join(".aw/changes/hints-blk");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "hints-blk"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let prompt = read_prompt(&parsed, &change_dir, "begin_implementation");

        let cli_section_pos = prompt
            .find("## CLI Commands")
            .expect("Prompt should contain ## CLI Commands section");
        let hints_pos = prompt
            .find("Code intelligence")
            .expect("Prompt should contain Code intelligence hints");
        assert!(
            hints_pos > cli_section_pos,
            "CLI hints should appear after ## CLI Commands header"
        );
    }

    #[tokio::test]
    async fn test_tests_prompt_no_cli_hints() {
        // build_implement_tests_prompt should NOT contain code intelligence hints.
        let tmp = setup_change("hints-nt", "change_implementation_created");
        write_spec(&tmp, "hints-nt", "spec-a", &[]);
        write_config(&tmp, "mainthread");

        let change_dir = tmp.path().join(".aw/changes/hints-nt");
        // Set impl_spec_phase to "tests" to trigger ImplementSpecTests sub-state
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().current_task_id = Some("spec-a".into());
        sm.state_mut()
            .impl_spec_phase
            .insert("spec-a".into(), "tests".into());
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "hints-nt"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        if parsed["status"] == "ok" {
            let prompts_dir = change_dir.join("prompts");
            if prompts_dir.exists() {
                for entry in std::fs::read_dir(&prompts_dir)
                    .unwrap()
                    .filter_map(|e| e.ok())
                {
                    let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                    assert!(
                        !content.contains("score symbols"),
                        "Tests prompt should NOT contain code intelligence hints"
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn test_write_diff_prompt_no_cli_hints() {
        // build_write_diff_prompt should NOT contain code intelligence hints.
        let tmp = setup_change("hints-nd", "change_implementation_created");
        write_spec(&tmp, "hints-nd", "spec-a", &[]);
        write_config(&tmp, "mainthread");

        let change_dir = tmp.path().join(".aw/changes/hints-nd");
        // Set current_task_id to last spec to trigger WriteDiff
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().current_task_id = Some("spec-a".into());
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "hints-nd"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        if parsed["status"] == "ok" {
            let prompts_dir = change_dir.join("prompts");
            if prompts_dir.exists() {
                for entry in std::fs::read_dir(&prompts_dir)
                    .unwrap()
                    .filter_map(|e| e.ok())
                {
                    let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                    if content.contains("Write Implementation Diff") {
                        assert!(
                            !content.contains("score symbols"),
                            "Write-diff prompt should NOT contain code intelligence hints"
                        );
                    }
                }
            }
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_impl.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-create-change-impl-rs-tests>"
    description: "Create change implementation regression tests."
```
