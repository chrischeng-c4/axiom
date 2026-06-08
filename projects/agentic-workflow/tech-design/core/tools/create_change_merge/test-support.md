---
id: sdd-tools-create-change-merge-test-support
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change merge test support

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_merge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 69 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod test_support {
    use super::*;
    use crate::state::StateManager;
    use tempfile::TempDir;

    pub(super) fn setup_change(change_id: &str, phase: StatePhase) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/tech-design")).unwrap();
        // R4: save() needs an issue backing change_id.
        crate::test_util::write_minimal_issue(tmp.path(), change_id);

        // Write minimal config.toml with required platform sections
        let config_content = r#"
[agentic_workflow.repo_platform]
type = "github"
repo = "test/repo"
default_branch = "main"
auto_commit = false
auto_pr = false

[agentic_workflow.tech_design_platform]
type = "local"
path = ".aw/tech-design"
"#;
        std::fs::write(tmp.path().join(".aw/config.toml"), config_content).unwrap();

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = phase;
        sm.save().unwrap();

        tmp
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_merge.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
    description: "Shared test setup helper for create-change-merge regression tests."
```
