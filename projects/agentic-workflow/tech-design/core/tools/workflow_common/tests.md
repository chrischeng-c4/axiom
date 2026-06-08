---
id: sdd-tools-workflow-common-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools workflow common tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/workflow_common.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `build_group_issues_hint` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 442 | build_group_issues_hint(change_dir: &Path, group_id: &str) -> String |
| `build_workflow_response` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 395 | build_workflow_response(     change_dir: &Path,     change_id: &str,     action: &str,     prompt: String,     executor: Vec<String>,     extra_fields: Value,     _interface: SddInterface,     _project_root: &Path, ) -> Result<String> |
| `get_executor_chain` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 273 | get_executor_chain(_project_root: &Path, artifact: WorkflowArtifact) -> Vec<String> |
| `has_uncommitted_diff` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 209 | has_uncommitted_diff(project_root: &Path, rel_path: &str) -> Result<bool> |
| `is_git_project` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 174 | is_git_project(project_root: &Path) -> bool |
| `is_git_tracked` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 192 | is_git_tracked(project_root: &Path, rel_path: &str) -> Result<bool> |
| `list_group_ids` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 325 | list_group_ids(groups_dir: &Path) -> Result<Vec<String>> |
| `load_interface` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 309 | load_interface(project_root: &Path) -> SddInterface |
| `next_action` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 319 | next_action(interface: SddInterface, tool: &str, args: Value) -> Value |
| `phase_to_string` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 144 | phase_to_string(phase: &StatePhase) -> &'static str |
| `resolve_active_change_id` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 76 | resolve_active_change_id(project_root: &Path) -> Result<String> |
| `resolve_change_dir` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 120 | resolve_change_dir(project_root: &Path, change_id: &str) -> PathBuf |
| `resolve_single_group_id` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 346 | resolve_single_group_id(change_dir: &Path) -> Option<String> |
| `update_phase` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 250 | update_phase(change_dir: &Path, phase: StatePhase) -> Result<()> |
| `validate_change_dir` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 39 | validate_change_dir(change_dir: &Path, project_root: &Path) -> Result<()> |
| `validate_change_id` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 18 | validate_change_id(change_id: &str) -> Result<()> |
| `write_prompt_file` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 364 | write_prompt_file(     change_dir: &Path,     group_id: Option<&str>,     action: &str,     prompt: &str, ) -> Result<PathBuf> |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    /// Create a minimal change directory structure for testing build_workflow_response.
    fn setup_change_dir(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(change_dir.join("prompts")).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        tmp
    }

    // ======================================================================
    // Subagent executor routing tests (sdd-subagent-mode)
    // ======================================================================

    #[tokio::test]
    async fn test_build_workflow_response_subagent_returns_prompt_path_and_executor() {
        // subagent:* executors should be returned to caller (like mainthread),
        // NOT dispatched via run_agent()
        let tmp = setup_change_dir("test-sub");
        let change_dir = tmp.path().join(".aw/changes/test-sub");

        let result = build_workflow_response(
            &change_dir,
            "test-sub",
            "create_reference_context",
            "Test prompt".to_string(),
            vec!["subagent:Explore".to_string()],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(
            parsed["prompt_path"]
                .as_str()
                .unwrap()
                .contains("create_reference_context"),
            "prompt_path should contain action name"
        );
        assert_eq!(
            parsed["executor"][0], "subagent:Explore",
            "executor should be passed through as-is"
        );
        // next_actions should be empty (no run_agent dispatch)
        assert_eq!(
            parsed["next_actions"].as_array().unwrap().len(),
            0,
            "subagent executors must have empty next_actions"
        );
    }

    #[tokio::test]
    async fn test_build_workflow_response_subagent_writes_prompt_file() {
        // Prompt must be written to file even for subagent executors
        let tmp = setup_change_dir("test-sub-prompt");
        let change_dir = tmp.path().join(".aw/changes/test-sub-prompt");

        let prompt_content = "Explore specs for auth module";
        build_workflow_response(
            &change_dir,
            "test-sub-prompt",
            "create_reference_context",
            prompt_content.to_string(),
            vec!["subagent:Explore".to_string()],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let prompt_file = change_dir.join("prompts/create_reference_context.md");
        assert!(prompt_file.exists(), "prompt file must be written");
        let written = std::fs::read_to_string(&prompt_file).unwrap();
        assert_eq!(written, prompt_content);
    }

    #[tokio::test]
    async fn test_build_workflow_response_subagent_with_group_id() {
        // When group_id is present, prompt goes to groups/{gid}/prompts/
        let tmp = setup_change_dir("test-sub-group");
        let change_dir = tmp.path().join(".aw/changes/test-sub-group");

        let result = build_workflow_response(
            &change_dir,
            "test-sub-group",
            "create_change_spec",
            "Write spec".to_string(),
            vec!["subagent:score-change-spec".to_string()],
            json!({"group_id": "auth-module"}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(
            parsed["prompt_path"]
                .as_str()
                .unwrap()
                .contains("groups/auth-module/prompts/"),
            "prompt_path should include group_id"
        );
        assert_eq!(parsed["executor"][0], "subagent:score-change-spec");

        // Verify group_id is merged into response
        assert_eq!(parsed["group_id"], "auth-module");

        // Verify prompt file written to group dir
        let prompt_file = change_dir.join("groups/auth-module/prompts/create_change_spec.md");
        assert!(prompt_file.exists(), "prompt file must be in group dir");
    }

    #[tokio::test]
    async fn test_build_workflow_response_mainthread_still_works() {
        // Mainthread executor should still work (regression test)
        let tmp = setup_change_dir("test-mt");
        let change_dir = tmp.path().join(".aw/changes/test-mt");

        let result = build_workflow_response(
            &change_dir,
            "test-mt",
            "restructure_input",
            "Restructure".to_string(),
            vec!["mainthread".to_string()],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["executor"][0], "mainthread");
        assert_eq!(
            parsed["next_actions"].as_array().unwrap().len(),
            0,
            "mainthread must have empty next_actions"
        );
    }

    #[tokio::test]
    async fn test_build_workflow_response_subagent_vs_mainthread_same_shape() {
        // Both subagent and mainthread produce the same response shape
        let tmp = setup_change_dir("test-shape");
        let change_dir = tmp.path().join(".aw/changes/test-shape");

        let subagent_result = build_workflow_response(
            &change_dir,
            "test-shape",
            "create_change_spec",
            "prompt1".to_string(),
            vec!["subagent:score-change-spec".to_string()],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let mt_result = build_workflow_response(
            &change_dir,
            "test-shape",
            "restructure_input",
            "prompt2".to_string(),
            vec!["mainthread".to_string()],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let sub_parsed: Value = serde_json::from_str(&subagent_result).unwrap();
        let mt_parsed: Value = serde_json::from_str(&mt_result).unwrap();

        // Both should have same top-level keys
        assert_eq!(sub_parsed["status"], mt_parsed["status"]);
        assert!(sub_parsed["prompt_path"].is_string());
        assert!(mt_parsed["prompt_path"].is_string());
        assert!(sub_parsed["executor"].is_array());
        assert!(mt_parsed["executor"].is_array());
        assert_eq!(
            sub_parsed["next_actions"].as_array().unwrap().len(),
            mt_parsed["next_actions"].as_array().unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_build_workflow_response_multiple_subagent_executors() {
        // Even with multiple subagent executors, should still return (not dispatch)
        let tmp = setup_change_dir("test-multi-sub");
        let change_dir = tmp.path().join(".aw/changes/test-multi-sub");

        let result = build_workflow_response(
            &change_dir,
            "test-multi-sub",
            "create_change_spec",
            "multi prompt".to_string(),
            vec![
                "subagent:score-change-spec".to_string(),
                "subagent:Explore".to_string(),
            ],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        // is_subagent is true because .any() matches first element
        assert_eq!(
            parsed["next_actions"].as_array().unwrap().len(),
            0,
            "multiple subagent executors should still return, not dispatch"
        );
    }

    // ======================================================================
    // get_executor_chain tests — hardcoded hybrid mapping (no config lookup)
    // ======================================================================

    #[test]
    fn test_get_executor_chain_is_pure_and_ignores_config() {
        // get_executor_chain is a pure function; it returns the same values
        // regardless of whether config.toml exists or what it contains.
        let tmp = TempDir::new().unwrap();
        let chain = get_executor_chain(tmp.path(), WorkflowArtifact::CreateReferenceContext);
        assert_eq!(chain, vec!["subagent:Explore".to_string()]);
    }

    #[test]
    fn test_get_executor_chain_hardcoded_mapping() {
        // Verify the hardcoded hybrid mapping: lightweight actions → mainthread,
        // heavy actions → subagent.
        let tmp = TempDir::new().unwrap();

        // Mainthread actions
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::RestructureInput),
            vec!["mainthread".to_string()]
        );
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::CreatePreClarifications),
            vec!["mainthread".to_string()]
        );
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::ReviseReferenceContext),
            vec!["mainthread".to_string()]
        );
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::CreateChangeMerge),
            vec!["mainthread".to_string()]
        );

        // Explore subagent (model defined in agent definition)
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::CreateReferenceContext),
            vec!["subagent:Explore".to_string()]
        );

        // Review subagents — quality gate (model defined in agent definition)
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::ReviewReferenceContext),
            vec!["subagent:score-review".to_string()]
        );
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::ReviewChangeSpec),
            vec!["subagent:score-review".to_string()]
        );
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::ReviewChangeImplementation),
            vec!["subagent:score-review".to_string()]
        );

        // Spec authoring (model defined in agent definition)
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::CreateChangeSpec),
            vec!["subagent:score-change-spec".to_string()]
        );

        // Impl authoring (model defined in agent definition)
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::CreateChangeImplementation),
            vec!["subagent:score-change-implementation".to_string()]
        );
    }

    // ======================================================================
    // is_subagent detection edge cases
    // ======================================================================

    #[test]
    fn test_subagent_prefix_must_be_exact() {
        // "subagent_foo" should NOT trigger subagent routing (missing colon)
        let executor = vec!["subagent_invalid".to_string()];
        let is_subagent = executor.iter().any(|e| e.starts_with("subagent:"));
        assert!(
            !is_subagent,
            "subagent_invalid should NOT be detected as subagent (no colon after 'subagent')"
        );

        // "sub" prefix should not match
        let executor2 = vec!["sub:agent:sonnet".to_string()];
        let is_subagent2 = executor2.iter().any(|e| e.starts_with("subagent:"));
        assert!(!is_subagent2, "sub:agent:sonnet must not match subagent:");
    }

    #[test]
    fn test_is_subagent_logic_matches_spec() {
        // Verify the detection logic from the spec's routing pseudocode
        let cases: Vec<(Vec<&str>, bool, bool)> = vec![
            // (executor, expected_is_mainthread_only, expected_is_subagent)
            (vec!["mainthread"], true, false),
            (vec!["subagent:Explore"], false, true),
            (vec!["subagent:score-change-spec"], false, true),
            (vec!["gemini:flash"], false, false),
            (vec!["claude-agent:change-spec"], false, false),
            (vec!["codex:balanced"], false, false),
        ];

        for (executor, exp_mt, exp_sub) in cases {
            let executor_strings: Vec<String> = executor.iter().map(|s| s.to_string()).collect();
            let is_mainthread_only =
                executor_strings.len() == 1 && executor_strings[0] == "mainthread";
            let is_subagent = executor_strings.iter().any(|e| e.starts_with("subagent:"));

            assert_eq!(
                is_mainthread_only, exp_mt,
                "executor {:?}: is_mainthread_only",
                executor
            );
            assert_eq!(is_subagent, exp_sub, "executor {:?}: is_subagent", executor);

            // When either is true, response should be returned (not dispatched)
            if exp_mt || exp_sub {
                assert!(
                    is_mainthread_only || is_subagent,
                    "executor {:?}: should be returned to caller",
                    executor
                );
            } else {
                assert!(
                    !is_mainthread_only && !is_subagent,
                    "executor {:?}: should be dispatched via run_agent()",
                    executor
                );
            }
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/workflow_common.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
      - "<module-trailer>"
    description: "Workflow-common response-shape and executor routing regression tests."
```
