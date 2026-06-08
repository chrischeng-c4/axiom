---
id: projects-sdd-src-tools-create-change-impl-rs-utils
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd create change implementation helpers

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
/// Run git command and return stdout (empty string on failure).
fn git_output(args: &[&str]) -> String {
    std::process::Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Auto-populate implementation.md with git diff baseline so the artifact
/// is never empty even if the agent fails downstream.
fn auto_populate_impl_baseline(change_id: &str, project_root: &Path) {
    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let impl_path = change_dir.join("implementation.md");
    // Don't overwrite if already has real content
    if impl_path.exists() {
        if let Ok(existing) = std::fs::read_to_string(&impl_path) {
            if existing.contains("## Diff") {
                return;
            }
        }
    }

    let stat = git_output(&["diff", "--stat", "main"]);
    let name_status = git_output(&["diff", "--name-status", "main"]);
    if stat.is_empty() && name_status.is_empty() {
        return;
    }

    let mut content = format!(
        "---\nid: implementation\ntype: change_implementation\nchange_id: {change_id}\n---\n\n\
         # Implementation\n\n\
         ## Summary\n\n*(auto-generated baseline from git diff)*\n\n"
    );

    if !name_status.is_empty() {
        content.push_str("## Changed Files\n\n```\n");
        content.push_str(&name_status);
        content.push_str("\n```\n\n");
    }

    if !stat.is_empty() {
        content.push_str("## Diff Statistics\n\n```\n");
        content.push_str(&stat);
        content.push_str("\n```\n\n");
    }

    // Include truncated diff
    let diff = git_output(&["diff", "main", "--", ".", ":!cclab/"]);
    if !diff.is_empty() {
        content.push_str("## Diff\n\n```diff\n");
        const MAX_LINES: usize = 2000;
        let lines: Vec<&str> = diff.lines().collect();
        if lines.len() > MAX_LINES {
            for line in &lines[..MAX_LINES] {
                content.push_str(line);
                content.push('\n');
            }
            content.push_str(&format!(
                "\n... truncated ({} more lines)\n",
                lines.len() - MAX_LINES
            ));
        } else {
            content.push_str(&diff);
            content.push('\n');
        }
        content.push_str("```\n");
    }

    let _ = std::fs::write(&impl_path, &content);
}

async fn build_write_diff_prompt(
    change_id: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    // Auto-populate baseline so impl is never empty if agent fails
    auto_populate_impl_baseline(change_id, project_root);

    let _pp = project_root.display();

    let prompt = format!(
        "# Task: Write Implementation Diff for Change '{change_id}'\n\n\
         ## Instructions\n\n\
         1. Run `git diff` (or `git diff HEAD~N..HEAD` if already committed) to get the full diff\n\
         2. Write `implementation.md` via the artifact CLI command\n\
         3. The artifact tool will redirect back to the workflow router automatically\n\n\
         ## CLI Commands\n\n\
         ```\n\
         # Write implementation artifact (write payload JSON first, then run)\n\
         score artifact create-change-implementation {change_id} .aw/changes/{change_id}/payloads/create-change-implementation.json\n\
         ```"
    );

    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreateChangeImplementation,
    );

    let mut extra = json!({});
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        "write_implementation_diff",
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Count `#[test]` occurrences in added lines of `git diff main`.
fn count_tests_in_diff(project_root: &Path) -> usize {
    let output = std::process::Command::new("git")
        .args(["diff", "main"])
        .current_dir(project_root)
        .output()
        .ok();

    let Some(out) = output else { return 0 };
    if !out.status.success() {
        return 0;
    }

    let diff = String::from_utf8_lossy(&out.stdout);
    diff.lines()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .filter(|line| line.contains("#[test]"))
        .count()
}

/// Parse the `## Unit Test` section of a spec and return the numeric test count if present.
///
/// Returns `Some(n)` if a markdown table with `n` data rows is found.
/// Returns `None` if the section is absent or has no numeric count (qualitative only).
/// Legacy `## Test Plan` sections are accepted while historical TDs migrate.
/// @spec projects/agentic-workflow/tech-design/core/tools/create_change_impl/utils.md#source
pub fn parse_test_plan_count(spec_content: &str) -> Option<usize> {
    // Find ## Unit Test section, falling back to legacy ## Test Plan.
    let test_plan_start = spec_content
        .find("## Unit Test")
        .or_else(|| spec_content.find("## Test Plan"))?;
    let after = &spec_content[test_plan_start..];

    // Find end: next ## heading or EOF
    let section_end = after[1..]
        .find("\n## ")
        .map(|i| i + 1)
        .unwrap_or(after.len());
    let section = &after[..section_end];

    // Count markdown table data rows (lines starting with `|` that are not separator rows `|---|`)
    let table_rows: usize = section
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with('|')
                && !trimmed
                    .chars()
                    .skip(1)
                    .all(|c| c == '-' || c == '|' || c == ' ' || c == ':')
        })
        .count();

    // Subtract 1 for the header row if we have at least 2 rows
    if table_rows >= 2 {
        Some(table_rows - 1)
    } else if table_rows == 1 {
        // Single row could be header only — no data rows
        None
    } else {
        // No table found — qualitative or absent
        None
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
      - "<handwrite-tracker:projects-sdd-src-tools-create-change-impl-rs-utils>"
    description: "Implementation git, diff, and test-count helpers."
```
