---
id: sdd-interfaces-services-path-scope-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Path Scope Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/path_scope.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AllowedScope` | projects/agentic-workflow/src/services/path_scope.rs | struct | pub | 61 |  |
| `ScopeProject` | projects/agentic-workflow/src/services/path_scope.rs | struct | pub | 40 |  |
| `ScopeWorkspace` | projects/agentic-workflow/src/services/path_scope.rs | struct | pub | 52 |  |
| `ScoreScopeConfig` | projects/agentic-workflow/src/services/path_scope.rs | struct | pub | 32 |  |
| `contains` | projects/agentic-workflow/src/services/path_scope.rs | function | pub | 108 | contains(&self, rel: &str) -> bool |
| `describe` | projects/agentic-workflow/src/services/path_scope.rs | function | pub | 119 | describe(&self) -> String |
| `for_project` | projects/agentic-workflow/src/services/path_scope.rs | function | pub | 74 | for_project(project: &ScopeProject) -> Result<Self> |
| `load_scope` | projects/agentic-workflow/src/services/path_scope.rs | function | pub | 134 | load_scope(root: &Path) -> Result<Option<ScoreScopeConfig>> |
| `project_by_name` | projects/agentic-workflow/src/services/path_scope.rs | function | pub | 148 | project_by_name(cfg: &'a ScoreScopeConfig, name: &str) -> Option<&'a ScopeProject> |
## Source
<!-- type: source lang: rust -->

```rust
//! Edit-scope resolution for the internal pretooluse write-scope hook.
//!
//! Reads `.aw/config.toml`, locates the `[[projects]]` entry by name,
//! and answers "is this relative path inside the project's allowed
//! edit scope?" (path prefix, td_path prefix, or any
//! `[[projects.workspaces]].paths` glob).
//!
//! Schema is intentionally narrower than `models::project::Project` —
//! we only deserialize the fields the hook needs (`name`, `path`,
//! `td_path`, `workspaces.paths`). Anything else (`label`, `target`,
//! `test_cmd`, `codegen`, …) is ignored. This decouples the hook from
//! `Project` schema churn and matches the original Python stopgap
//! exactly.
//!
//! @spec projects/agentic-workflow/tech-design/core/specs/score-hook-pretooluse-write-scope.md#logic
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/path_scope.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
    description: "Source template owns path-scope module documentation."
```
