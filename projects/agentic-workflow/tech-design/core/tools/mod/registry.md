---
id: sdd-tools-mod-registry
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools module registry

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ToolDefinition` | projects/agentic-workflow/src/tools/mod.rs | struct | pub | 66 |  |
| `ToolRegistry` | projects/agentic-workflow/src/tools/mod.rs | struct | pub | 77 |  |
| `analyze` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 8 |  |
| `artifact_read` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 9 |  |
| `artifact_write` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 10 |  |
| `call_tool` | projects/agentic-workflow/src/tools/mod.rs | function | pub | 238 | call_tool(&self, name: &str, arguments: &Value) -> Result<String> |
| `call_tool_streaming` | projects/agentic-workflow/src/tools/mod.rs | function | pub | 377 | call_tool_streaming(         &self,         name: &str,         arguments: &Value,         _tx: Option<mpsc::Sender<String>>,     ) -> Result<String> |
| `clarifications` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 11 |  |
| `common_change_impl` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 12 |  |
| `common_change_spec` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 13 |  |
| `common_reference_context` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 14 |  |
| `context` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 15 |  |
| `create_change_docs` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 16 |  |
| `create_change_impl` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 17 |  |
| `create_change_merge` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 18 |  |
| `create_change_spec` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 19 |  |
| `create_post_clarifications` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 20 |  |
| `create_pre_clarifications` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 21 |  |
| `create_reference_context` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 22 |  |
| `fetch_issues` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 41 |  |
| `fill_issue_reference_context` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 23 |  |
| `get_optional_string` | projects/agentic-workflow/src/tools/mod.rs | function | pub | 410 | get_optional_string(args: &Value, field: &str) -> Option<String> |
| `get_required_array` | projects/agentic-workflow/src/tools/mod.rs | function | pub | 418 | get_required_array(args: &Value, field: &str) -> Result<Vec<Value>> |
| `get_required_object` | projects/agentic-workflow/src/tools/mod.rs | function | pub | 427 | get_required_object(args: &Value, field: &str) -> Result<Value> |
| `get_required_string` | projects/agentic-workflow/src/tools/mod.rs | function | pub | 396 | get_required_string(args: &Value, field: &str) -> Result<String> |
| `implementation` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 37 |  |
| `init_change` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 24 |  |
| `knowledge` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 38 |  |
| `list_tools` | projects/agentic-workflow/src/tools/mod.rs | function | pub | 219 | list_tools(&self) -> Vec<Value> |
| `merge_git_ops` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 25 |  |
| `new` | projects/agentic-workflow/src/tools/mod.rs | function | pub | 88 | new() -> Self |
| `new_for_stage` | projects/agentic-workflow/src/tools/mod.rs | function | pub | 93 | new_for_stage(stage: &str) -> Self |
| `phase_transition` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 39 |  |
| `platform_sync` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 42 |  |
| `read` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 45 |  |
| `resolve_project_path` | projects/agentic-workflow/src/tools/mod.rs | function | pub | 468 | resolve_project_path(args: &Value) -> Result<PathBuf> |
| `review` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 26 |  |
| `review_change_docs` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 27 |  |
| `review_change_impl` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 28 |  |
| `review_change_spec` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 29 |  |
| `review_helpers` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 46 |  |
| `review_reference_context` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 30 |  |
| `revise_change_docs` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 31 |  |
| `revise_change_impl` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 32 |  |
| `revise_change_spec` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 33 |  |
| `revise_reference_context` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 34 |  |
| `spec` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 47 |  |
| `spec_plan` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 35 |  |
| `task` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 48 |  |
| `validate` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 49 |  |
| `validate_proposal` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 50 |  |
| `validate_spec` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 51 |  |
| `workflow_common` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 43 |  |
| `workflow_validate` | projects/agentic-workflow/src/tools/mod.rs | module | pub | 52 |  |
## Source
<!-- type: source lang: rust -->

````rust
impl ToolRegistry {
    /// Create a new tool registry with all available tools
    pub fn new() -> Self {
        Self::all_tools()
    }

    /// Create a tool registry filtered by workflow stage
    pub fn new_for_stage(stage: &str) -> Self {
        let tools = match stage {
            "plan" => Self::plan_tools(),
            "challenge" => Self::challenge_tools(),
            "implement" => Self::implement_tools(),
            "review" => Self::review_tools(),
            "archive" => Self::archive_tools(),
            _ => Self::all_tools_vec(),
        };
        Self { tools }
    }

    /// All tools (consolidated: sdd_write_artifact + sdd_read_artifact replace ~15 tools)
    ///
    /// Note: Mermaid and API spec generation tools have been moved to cclab-lens.
    /// Use lens_generate_* tools instead.
    fn all_tools() -> Self {
        Self {
            tools: Self::all_tools_vec(),
        }
    }

    fn all_tools_vec() -> Vec<ToolDefinition> {
        vec![
            // Standalone init change (#632)
            init_change::definition(),
            // Pre-clarifications workflow + artifact
            create_pre_clarifications::workflow_definition(),
            create_pre_clarifications::artifact_definition(),
            // Post-clarifications workflow + artifact (2 tools)
            create_post_clarifications::workflow_definition(),
            create_post_clarifications::artifact_definition(),
            // Reference context workflow + artifact (6 tools)
            create_reference_context::workflow_definition(),
            create_reference_context::artifact_definition(),
            review_reference_context::workflow_definition(),
            review_reference_context::artifact_definition(),
            revise_reference_context::workflow_definition(),
            revise_reference_context::artifact_definition(),
            // Change spec workflow + artifact (6 tools)
            create_change_spec::workflow_definition(),
            create_change_spec::artifact_definition(),
            review_change_spec::workflow_definition(),
            review_change_spec::artifact_definition(),
            revise_change_spec::workflow_definition(),
            revise_change_spec::artifact_definition(),
            // Change implementation workflow + artifact (6 tools)
            create_change_impl::workflow_definition(),
            create_change_impl::artifact_definition(),
            review_change_impl::workflow_definition(),
            review_change_impl::artifact_definition(),
            revise_change_impl::workflow_definition(),
            revise_change_impl::artifact_definition(),
            // Change docs workflow + artifact (6 tools)
            create_change_docs::workflow_definition(),
            create_change_docs::artifact_definition(),
            review_change_docs::workflow_definition(),
            review_change_docs::artifact_definition(),
            revise_change_docs::workflow_definition(),
            revise_change_docs::artifact_definition(),
            // Change merge (1 programmatic tool)
            create_change_merge::workflow_definition(),
            // Unified artifact tools (replace ~15 dedicated tools)
            artifact_write::definition(),
            artifact_read::definition(),
            // Workflow orchestration
            crate::workflow::definition(),
            validate::definition(),
            workflow_validate::definition(),
            // Implementation (read-only — not replaced by artifact_write)
            implementation::read_implementation_summary_definition(),
            implementation::list_changed_files_definition(),
            // Code analysis for spec generation
            analyze::definition(),
            // Spec validation
            validate_spec::definition(),
            // Platform sync
            platform_sync::definition(),
        ]
    }

    /// Plan stage tools (21 tools: all core)
    /// Used by: Gemini for proposal generation
    /// Note: For diagram/spec generation, use lens_generate_* tools
    fn plan_tools() -> Vec<ToolDefinition> {
        Self::all_tools_vec()
    }

    /// Challenge stage tools
    /// Used by: Codex for challenging proposals
    fn challenge_tools() -> Vec<ToolDefinition> {
        vec![
            artifact_read::definition(),
            artifact_write::definition(),
            validate::definition(),
        ]
    }

    /// Implement stage tools
    /// Used by: Claude for code implementation
    fn implement_tools() -> Vec<ToolDefinition> {
        vec![
            artifact_read::definition(),
            implementation::read_implementation_summary_definition(),
            implementation::list_changed_files_definition(),
        ]
    }

    /// Review stage tools
    /// Used by: Codex for code review
    fn review_tools() -> Vec<ToolDefinition> {
        vec![
            validate::definition(),
            artifact_read::definition(),
            artifact_write::definition(),
            implementation::read_implementation_summary_definition(),
        ]
    }

    /// Archive stage tools
    /// Used by: Gemini for merging specs to knowledge base
    fn archive_tools() -> Vec<ToolDefinition> {
        vec![artifact_read::definition(), artifact_write::definition()]
    }

    /// List all available tools in MCP format
    pub fn list_tools(&self) -> Vec<Value> {
        self.tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            })
            .collect()
    }

    /// Call a tool by name with the given arguments
    ///
    /// The project_path is extracted from the arguments for all tools.
    ///
    /// Note: Mermaid and API spec generation tools are in the generate submodule.
    /// Use sdd_generate_* tools instead (e.g., sdd_generate_flowchart, sdd_generate_openapi).
    pub async fn call_tool(&self, name: &str, arguments: &Value) -> Result<String> {
        // All tools require project_path
        let project_root = resolve_project_path(arguments)?;
        // Resolve payload_path: read JSON from file, merge with inline params
        let arguments = &resolve_payload(arguments, &project_root)?;

        match name {
            // Standalone init change (#632)
            "sdd_workflow_init_change" => init_change::execute_standalone(arguments, &project_root),
            // Pre-clarifications (group-aware)
            "sdd_workflow_create_pre_clarifications" => {
                create_pre_clarifications::execute_workflow_pre_clarifications(
                    arguments,
                    &project_root,
                )
                .await
            }
            "sdd_artifact_create_pre_clarifications" => {
                create_pre_clarifications::execute_artifact_pre_clarifications(
                    arguments,
                    &project_root,
                )
            }
            // Post-clarifications (group-aware, 2 tools)
            "sdd_workflow_create_post_clarifications" => {
                create_post_clarifications::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_create_post_clarifications" => {
                create_post_clarifications::execute_artifact(arguments, &project_root)
            }
            // Reference context (group-aware, 6 tools)
            "sdd_workflow_create_reference_context" => {
                create_reference_context::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_create_reference_context" => {
                create_reference_context::execute_artifact(arguments, &project_root)
            }
            "sdd_workflow_review_reference_context" => {
                review_reference_context::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_review_reference_context" => {
                review_reference_context::execute_artifact(arguments, &project_root)
            }
            "sdd_workflow_revise_reference_context" => {
                revise_reference_context::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_revise_reference_context" => {
                revise_reference_context::execute_artifact(arguments, &project_root)
            }
            // Change spec (6 tools)
            "sdd_workflow_create_change_spec" => {
                create_change_spec::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_create_change_spec" => {
                create_change_spec::execute_artifact(arguments, &project_root)
            }
            "sdd_workflow_review_change_spec" => {
                review_change_spec::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_review_change_spec" => {
                review_change_spec::execute_artifact(arguments, &project_root)
            }
            "sdd_workflow_revise_change_spec" => {
                revise_change_spec::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_revise_change_spec" => {
                revise_change_spec::execute_artifact(arguments, &project_root)
            }
            // Change implementation (6 tools)
            "sdd_workflow_create_change_implementation" => {
                create_change_impl::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_create_change_implementation" => {
                create_change_impl::execute_artifact(arguments, &project_root)
            }
            "sdd_workflow_review_change_implementation" => {
                review_change_impl::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_review_change_implementation" => {
                review_change_impl::execute_artifact(arguments, &project_root)
            }
            "sdd_workflow_revise_change_implementation" => {
                revise_change_impl::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_revise_change_implementation" => {
                revise_change_impl::execute_artifact(arguments, &project_root)
            }
            // Change docs (6 tools)
            "sdd_workflow_create_change_docs" => {
                create_change_docs::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_create_change_docs" => {
                create_change_docs::execute_artifact(arguments, &project_root)
            }
            "sdd_workflow_review_change_docs" => {
                review_change_docs::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_review_change_docs" => {
                review_change_docs::execute_artifact(arguments, &project_root)
            }
            "sdd_workflow_revise_change_docs" => {
                revise_change_docs::execute_workflow(arguments, &project_root).await
            }
            "sdd_artifact_revise_change_docs" => {
                revise_change_docs::execute_artifact(arguments, &project_root)
            }
            // Change merge (1 programmatic tool)
            "sdd_workflow_create_change_merge" => {
                create_change_merge::execute_workflow(arguments, &project_root).await
            }
            // Unified artifact tools
            "sdd_write_artifact" => artifact_write::execute(arguments, &project_root),
            "sdd_read_artifact" => artifact_read::execute(arguments, &project_root),
            // Workflow orchestration
            "sdd_run_change" => crate::workflow::execute(arguments, &project_root).await,
            "sdd_validate_change" => validate::execute(arguments, &project_root).await,
            "sdd_workflow_validate" => workflow_validate::execute(arguments, &project_root).await,
            // Implementation (read-only)
            "sdd_read_implementation_summary" => {
                implementation::execute_read_implementation_summary(arguments, &project_root)
            }
            "sdd_list_changed_files" => {
                implementation::execute_list_changed_files(arguments, &project_root)
            }
            "sdd_analyze_code_for_spec" => analyze::execute(arguments, &project_root),
            "sdd_validate_spec_completeness" => validate_spec::execute(arguments, &project_root),
            // Platform sync
            "sdd_platform_sync" => platform_sync::execute(arguments, &project_root).await,
            _ => anyhow::bail!("Unknown tool: {}", name),
        }
    }

    /// Call a tool by name with optional streaming channel.
    ///
    /// Score no longer has any tools that stream subprocess output. The
    /// `sdd_delegate_agent` tool + subprocess runner were deleted when Score
    /// moved to client-dispatched executor model (Claude Code subagent only).
    /// This method now just forwards to `call_tool`, ignoring `_tx`. Kept as a
    /// stable API surface for cclab-server's streaming path.
    pub async fn call_tool_streaming(
        &self,
        name: &str,
        arguments: &Value,
        _tx: Option<mpsc::Sender<String>>,
    ) -> Result<String> {
        self.call_tool(name, arguments).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to extract a required string field from JSON
pub fn get_required_string(args: &Value, field: &str) -> Result<String> {
    let value = args
        .get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing required field: {}", field))?;
    if value.is_empty() {
        anyhow::bail!("Required field '{}' cannot be empty", field);
    }
    Ok(value)
}

/// Helper to extract an optional string field from JSON
pub fn get_optional_string(args: &Value, field: &str) -> Option<String> {
    args.get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Helper to extract a required array field from JSON
pub fn get_required_array(args: &Value, field: &str) -> Result<Vec<Value>> {
    args.get(field)
        .and_then(|v| v.as_array())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Missing required array field: {}", field))
}

/// Helper to extract a required object field from JSON
pub fn get_required_object(args: &Value, field: &str) -> Result<Value> {
    args.get(field)
        .filter(|v| v.is_object())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Missing required object field: {}", field))
}

/// Resolve `payload_path` from tool arguments.
///
/// If `payload_path` is present, reads the JSON file and merges with inline params.
/// Direct params override payload values (except `payload_path` itself).
fn resolve_payload(args: &Value, project_root: &Path) -> Result<Value> {
    let payload_path = match args.get("payload_path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return Ok(args.clone()),
    };
    let path = if payload_path.starts_with('/') {
        PathBuf::from(payload_path)
    } else {
        project_root.join(payload_path)
    };
    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read payload_path '{}': {}", path.display(), e))?;
    let mut payload: Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in payload_path '{}': {}", path.display(), e))?;
    // Direct params override payload values
    if let (Some(payload_obj), Some(args_obj)) = (payload.as_object_mut(), args.as_object()) {
        for (k, v) in args_obj {
            if k != "payload_path" {
                payload_obj.insert(k.clone(), v.clone());
            }
        }
    }
    Ok(payload)
}

/// Resolve project_path from tool arguments
///
/// Extracts and validates the project_path parameter from MCP tool arguments.
/// Supports ~ expansion for home directory.
pub fn resolve_project_path(args: &Value) -> Result<PathBuf> {
    let path_str = get_required_string(args, "project_path")?;

    // Expand ~ to home directory
    let expanded = if path_str.starts_with("~") {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;
        path_str.replacen("~", &home, 1)
    } else {
        path_str
    };

    let path = PathBuf::from(&expanded);

    // Validate path exists and has .aw workspace directory
    let ws = path.join(crate::shared::workspace::WORKSPACE_DIR);
    if !ws.exists() {
        anyhow::bail!(
            "Not a Score project: {} ({} directory not found). Run `aw init` first.",
            path.display(),
            crate::shared::workspace::WORKSPACE_DIR
        );
    }

    Ok(path)
}

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "ToolRegistry construction, dispatch, payload resolution, and project path helpers."
```
