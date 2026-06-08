---
id: sdd-tools-workflow-common-group-issues-hint
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools workflow common group issues hint

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
/// Build a hint string listing issues belonging to a group (from frontmatter).
pub fn build_group_issues_hint(change_dir: &Path, group_id: &str) -> String {
    let issues_dir = change_dir.join("issues");
    if !issues_dir.exists() {
        return String::new();
    }

    let mut hints = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&issues_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("issue_") && name.ends_with(".md") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if content.contains(&format!("group: \"{}\"", group_id)) {
                        if let Some(num_str) = name
                            .strip_prefix("issue_")
                            .and_then(|s| s.strip_suffix(".md"))
                        {
                            hints.push(format!("#{}", num_str));
                        }
                    }
                }
            }
        }
    }
    hints.join(", ")
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
      - "build_group_issues_hint"
    description: "Group issue hint builder from issue frontmatter."
```
