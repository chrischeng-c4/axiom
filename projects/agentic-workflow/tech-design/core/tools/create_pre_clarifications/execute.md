---
id: sdd-tools-create-pre-clarifications-execute
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create pre clarifications execute

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_pre_clarifications.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 287 | artifact_definition() -> ToolDefinition |
| `definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 17 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 70 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_append` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 199 | execute_append(args: &Value, project_root: &Path) -> Result<String> |
| `execute_artifact_pre_clarifications` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 418 | execute_artifact_pre_clarifications(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow_pre_clarifications` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 343 | execute_workflow_pre_clarifications(     args: &Value,     project_root: &Path, ) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 262 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let questions = get_required_array(args, "questions")?;

    // Validate change_id format (security: prevent directory traversal)
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }

    // Create change directory if needed
    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }

    let clarifications_path = change_dir.join("pre_clarifications.md");

    // Check for DAG in STATE.yaml (for multi-issue frontmatter)
    let dag = if change_dir.join("STATE.yaml").exists() {
        StateManager::load(&change_dir)
            .ok()
            .and_then(|sm| sm.state().dag.clone())
    } else {
        None
    };

    // Generate frontmatter (include issues list when DAG exists)
    let today = Local::now().format("%Y-%m-%d").to_string();
    let issues_line = if let Some(ref dag) = dag {
        let nums: Vec<String> = dag.issues.iter().map(|i| i.number.to_string()).collect();
        format!("\nissues: [{}]", nums.join(", "))
    } else {
        String::new()
    };
    let mut content = format!(
        "---\nchange: {}\ndate: {}{}\n---\n\n# Context Clarifications\n\n",
        change_id, today, issues_line
    );

    // Generate Q&A sections
    for (i, qa) in questions.iter().enumerate() {
        let topic = qa
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("General");
        let question = qa.get("question").and_then(|v| v.as_str()).unwrap_or("");
        let answer = qa.get("answer").and_then(|v| v.as_str()).unwrap_or("");
        let rationale = qa.get("rationale").and_then(|v| v.as_str()).unwrap_or("");

        content.push_str(&format!("## Q{}: {}\n", i + 1, topic));
        content.push_str(&format!("- **Question**: {}\n", question));
        content.push_str(&format!("- **Answer**: {}\n", answer));
        content.push_str(&format!("- **Rationale**: {}\n", rationale));
        content.push('\n');
    }

    // Inject dependency graph section when DAG exists
    if let Some(ref dag) = dag {
        if !dag.issues.is_empty() {
            content.push_str("## Dependency Graph\n\n");
            content.push_str("| Order | Issue | Depends On |\n");
            content.push_str("|-------|-------|------------|\n");
            for (i, issue) in dag.issues.iter().enumerate() {
                let deps = if issue.blocked_by.is_empty() {
                    "\u{2014}".to_string()
                } else {
                    issue
                        .blocked_by
                        .iter()
                        .map(|d| format!("#{}", d))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                content.push_str(&format!(
                    "| {} | #{} \u{2014} {} | {} |\n",
                    i + 1,
                    issue.number,
                    issue.title,
                    deps
                ));
            }
            content.push('\n');

            // Mermaid diagram
            content.push_str("```mermaid\ngraph LR\n");
            for issue in &dag.issues {
                content.push_str(&format!(
                    "    {}[\"#{} {}\"]\n",
                    issue.number, issue.number, issue.title
                ));
            }
            for issue in &dag.issues {
                for dep in &issue.blocked_by {
                    content.push_str(&format!("    {} --> {}\n", dep, issue.number));
                }
            }
            content.push_str("```\n\n");
        }
    }

    std::fs::write(&clarifications_path, content)?;

    // Auto-update STATE.yaml phase
    super::workflow_common::update_phase(&change_dir, StatePhase::ChangeInited)?;

    let result = json!({
        "status": "ok",
        "change_id": change_id,
        "artifacts": [
            format!(".aw/changes/{}/pre_clarifications.md", change_id),
            format!(".aw/changes/{}/STATE.yaml", change_id),
        ],
        "phase": "change_inited",
        "questions_count": questions.len(),
        "next": "sdd_run_change"
    });
    Ok(result.to_string())
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_pre_clarifications.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute"
    description: "Legacy pre-clarifications artifact writer and phase update."
```
