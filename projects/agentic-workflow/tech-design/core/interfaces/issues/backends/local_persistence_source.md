---
id: sdd-interfaces-issues-backends-local-persistence-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Local Backend Persistence Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/backends/local.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `LocalBackend` | projects/agentic-workflow/src/issues/backends/local.rs | struct | pub | 41 |  |
| `at` | projects/agentic-workflow/src/issues/backends/local.rs | function | pub | 58 | at(issues_dir: PathBuf) -> Self |
| `from_project_root` | projects/agentic-workflow/src/issues/backends/local.rs | function | pub | 52 | from_project_root(project_root: &Path) -> Self |
| `issue_path` | projects/agentic-workflow/src/issues/backends/local.rs | function | pub | 68 | issue_path(&self, issue: &Issue) -> PathBuf |
| `issues_dir` | projects/agentic-workflow/src/issues/backends/local.rs | function | pub | 63 | issues_dir(&self) -> &Path |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap local-backend-frontmatter-conversion -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/backends/local.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:local-backend-frontmatter-conversion>"
    description: "Source template owns local backend frontmatter conversion and persistence helpers."
```
