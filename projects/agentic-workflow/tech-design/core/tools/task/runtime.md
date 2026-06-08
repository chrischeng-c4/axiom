---
id: sdd-tools-task-runtime
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools task runtime

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/task.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TaskType` | projects/agentic-workflow/src/tools/task.rs | enum | pub | 20 |  |
| `definition` | projects/agentic-workflow/src/tools/task.rs | function | pub | 55 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/task.rs | function | pub | 114 | execute(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
impl TaskType {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "explore" => Ok(TaskType::Explore),
            "create_spec" => Ok(TaskType::CreateSpec),
            "review_spec" => Ok(TaskType::ReviewSpec),
            "revise_spec" => Ok(TaskType::ReviseSpec),
            "revise_tasks" => Ok(TaskType::ReviseTasks),
            "implement" => Ok(TaskType::Implement),
            "code_review" => Ok(TaskType::CodeReview),
            "resolve" => Ok(TaskType::Resolve),
            "review_archive" => Ok(TaskType::ReviewArchive),
            _ => anyhow::bail!("Unknown task_type: {}", s),
        }
    }
}

/// Get the tool definition for get_task
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_get_task".to_string(),
        description: "Get task instructions for the current workflow step. Call this first to understand your assignment.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "task_type"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "description": "The change ID to work on"
                },
                "task_type": {
                    "type": "string",
                    "enum": [
                        "explore",
                        "create_spec",
                        "review_spec",
                        "revise_spec",
                        "revise_tasks",
                        "implement",
                        "code_review",
                        "resolve",
                        "review_archive"
                    ],
                    "description": "The type of task to perform"
                },
                "spec_id": {
                    "type": "string",
                    "description": "Spec ID (required for create_spec, review_spec, revise_spec tasks)"
                },
                "description": {
                    "type": "string",
                    "description": "User's description of the change"
                },
                "iteration": {
                    "type": "integer",
                    "description": "Current iteration number (for review_* tasks)"
                },
                "dependencies": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of spec IDs this spec depends on (for create_spec task)"
                },
                "strategy": {
                    "type": "string",
                    "description": "Archive strategy (for review_archive task)"
                }
            }
        }),
    }
}

/// Execute the get_task tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let task_type_str = get_required_string(args, "task_type")?;
    let task_type = TaskType::from_str(&task_type_str)?;

    let spec_id = get_optional_string(args, "spec_id");
    let description = get_optional_string(args, "description");
    let strategy = get_optional_string(args, "strategy");
    let iteration = args.get("iteration").and_then(|v| v.as_i64()).unwrap_or(1);
    let dependencies: Vec<String> = args
        .get("dependencies")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Build variables map
    let mut vars = HashMap::new();
    vars.insert(
        "project_path".to_string(),
        project_root.display().to_string(),
    );
    vars.insert("change_id".to_string(), change_id.clone());
    vars.insert("iteration".to_string(), iteration.to_string());

    if let Some(sid) = &spec_id {
        vars.insert("spec_id".to_string(), sid.clone());
    }
    if let Some(desc) = &description {
        vars.insert("description".to_string(), desc.clone());
    }
    if let Some(strat) = &strategy {
        vars.insert("strategy".to_string(), strat.clone());
    }
    // Always include today's date for review_archive
    vars.insert(
        "today".to_string(),
        Local::now().format("%Y-%m-%d").to_string(),
    );

    // Format dependencies for template
    if dependencies.is_empty() {
        vars.insert("dependencies".to_string(), "None".to_string());
    } else {
        let deps_list = dependencies
            .iter()
            .map(|d| format!("- `{}`", d))
            .collect::<Vec<_>>()
            .join("\n");
        vars.insert("dependencies".to_string(), deps_list);
    }

    // Validate required variables for specific task types
    match task_type {
        TaskType::CreateSpec | TaskType::ReviewSpec | TaskType::ReviseSpec => {
            if spec_id.is_none() {
                anyhow::bail!("spec_id is required for {} task", task_type_str);
            }
        }
        TaskType::ReviewArchive => {
            if strategy.is_none() {
                vars.insert("strategy".to_string(), "standard".to_string());
            }
        }
        _ => {}
    }

    // Get embedded template and render
    let template = get_template(task_type);
    let rendered = render_template(template, &vars);

    Ok(rendered)
}

/// Get embedded template for task type
fn get_template(task_type: TaskType) -> &'static str {
    match task_type {
        TaskType::Explore => include_str!("../prompts/explore.md"),
        TaskType::CreateSpec => include_str!("../prompts/create_spec.md"),
        TaskType::ReviewSpec => include_str!("../prompts/review_spec.md"),
        TaskType::ReviseSpec => include_str!("../prompts/revise_spec.md"),
        TaskType::ReviseTasks => include_str!("../prompts/revise_tasks.md"),
        TaskType::Implement => include_str!("../prompts/implement.md"),
        TaskType::CodeReview => include_str!("../prompts/code_review.md"),
        TaskType::Resolve => include_str!("../prompts/resolve.md"),
        TaskType::ReviewArchive => include_str!("../prompts/review_archive.md"),
    }
}

/// Render template by replacing {{variable}} placeholders
fn render_template(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_render_template() {
        let template = "Change: {{change_id}}, Spec: {{spec_id}}";
        let mut vars = HashMap::new();
        vars.insert("change_id".to_string(), "my-feature".to_string());
        vars.insert("spec_id".to_string(), "auth-flow".to_string());

        let rendered = render_template(template, &vars);
        assert_eq!(rendered, "Change: my-feature, Spec: auth-flow");
    }

    #[test]
    fn test_get_task_spec_requires_spec_id() {
        let temp_dir = TempDir::new().unwrap();
        let args = json!({
            "change_id": "test",
            "task_type": "create_spec"
        });

        let result = execute(&args, temp_dir.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("spec_id is required"));
    }

    #[test]
    fn test_get_template_returns_embedded_prompts() {
        // Verify that get_template returns non-empty content for all task types
        assert!(!get_template(TaskType::Explore).is_empty());
        assert!(!get_template(TaskType::CreateSpec).is_empty());
        assert!(!get_template(TaskType::ReviewSpec).is_empty());
        assert!(!get_template(TaskType::ReviseSpec).is_empty());
        assert!(!get_template(TaskType::ReviseTasks).is_empty());
        assert!(!get_template(TaskType::Implement).is_empty());
        assert!(get_template(TaskType::CodeReview).contains("Code Review"));
        assert!(!get_template(TaskType::Resolve).is_empty());
        assert!(!get_template(TaskType::ReviewArchive).is_empty());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/task.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Runtime implementation for the get-task MCP tool."
```
