---
id: sdd-interfaces-issues-backends-local-tests-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: agent-first-cli-product-model
    claim: agent-first-cli-product-model
    coverage: full
    rationale: "Issue backend interfaces project the single AW CLI workflow state to configured issue platforms."
---

# Local Backend Tests Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/issues/backends/local.rs` generated from AST during Score force-regeneration standardization. Local compatibility fixtures read each legacy non-epic frontmatter value as canonical `change` while proving the original file and label remain unmodified. Canonical `type: change` cache entries are also parsed together so one valid cached Change cannot poison lookup of an unrelated lifecycle target (#2772).

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `LocalBackend` | apps/agentic-workflow/src/issues/backends/local.rs | struct | pub | 41 |  |
| `at` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 58 | at(issues_dir: PathBuf) -> Self |
| `from_project_root` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 52 | from_project_root(project_root: &Path) -> Self |
| `issue_path` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 68 | issue_path(&self, issue: &Issue) -> PathBuf |
| `issues_dir` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 63 | issues_dir(&self) -> &Path |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap local-backend-tests -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/issues/backends/local.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:local-backend-tests>"
    description: "Source template owns local backend regression tests, including lossless legacy non-epic compatibility reads and canonical Change cache isolation for #2772."
```
