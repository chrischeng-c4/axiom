---
id: sdd-interfaces-issues-module-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Issues Module Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/issues/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `backend` | apps/agentic-workflow/src/issues/mod.rs | module | pub | 25 |  |
| `backends` | apps/agentic-workflow/src/issues/mod.rs | module | pub | 26 |  |
| `github_backend` | apps/agentic-workflow/src/issues/mod.rs | function | pub | 238 | github_backend() -> GitHubBackend |
| `labels` | apps/agentic-workflow/src/issues/mod.rs | module | pub | 27 |  |
| `local_backend` | apps/agentic-workflow/src/issues/mod.rs | function | pub | 184 | local_backend(project_root: &Path) -> LocalBackend |
| `make_backend` | apps/agentic-workflow/src/issues/mod.rs | function | pub | 57 | make_backend(     kind: &str,     project_root: &Path,     repo: Option<String>,     host: Option<String>, ) -> Result<Box<dyn IssueBackend>> |
| `next_id` | apps/agentic-workflow/src/issues/mod.rs | module | pub | 28 |  |
| `push_through` | apps/agentic-workflow/src/issues/mod.rs | module | pub | 29 |  |
| `remote_read_cache_backend` | apps/agentic-workflow/src/issues/mod.rs | function | pub | 207 | remote_read_cache_backend(     kind: &str,     repo: Option<&str>,     host: Option<&str>, ) -> LocalBackend |
| `remote_read_cache_dir` | apps/agentic-workflow/src/issues/mod.rs | function | pub | 192 | remote_read_cache_dir(kind: &str, repo: Option<&str>, host: Option<&str>) -> PathBuf |
| `resolve_default_backend` | apps/agentic-workflow/src/issues/mod.rs | function | pub | 87 | resolve_default_backend(     project_root: &Path, ) -> Result<(String, Option<String>, Option<String>)> |
| `slug` | apps/agentic-workflow/src/issues/mod.rs | module | pub | 30 |  |
| `types` | apps/agentic-workflow/src/issues/mod.rs | module | pub | 31 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap issues-module-facade-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/issues/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:issues-module-facade-runtime>"
    description: "Source template owns the issues module facade runtime and tests."
```
