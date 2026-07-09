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

Public API manifest for `apps/agentic-workflow/src/tools/create_change_merge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `execute_workflow` | apps/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 69 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | apps/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/tools/create_change_merge/test-support.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod test_support {
    use super::*;
    use crate::state::StateManager;
    use tempfile::TempDir;

    pub(super) fn setup_change(change_id: &str, phase: StatePhase) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = crate::shared::workspace::change_path(tmp.path(), change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(tmp.path().join("tech-design")).unwrap();
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
path = "tech-design"
"#;
        std::fs::write(tmp.path().join("aw.toml"), config_content).unwrap();

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = phase;
        sm.save().unwrap();

        tmp
    }
}
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/tools/create_change_merge.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
    description: "Shared test setup helper for create-change-merge regression tests."
```
