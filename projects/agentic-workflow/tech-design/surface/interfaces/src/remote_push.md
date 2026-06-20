---
id: projects-score-src-remote-push-rs
fill_sections: [overview, changes]
capability_refs:
  - id: work-item-planning
    role: primary
    gap: capability-to-epic-planning
    claim: capability-to-epic-planning
    coverage: full
    rationale: "Issue/update CLI surfaces support work-item planning, projection, and platform synchronization."
---

# Standardized projects/agentic-workflow/src/cli/remote_push.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/remote_push.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `maybe_push_remote` | projects/agentic-workflow/src/cli/remote_push.rs | function | pub | 27 | maybe_push_remote(project_root: &Path, issue_path: &Path, slug: &str) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/remote_push.rs -->
```rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/remote_push.md#source
// CODEGEN-BEGIN

//! Shared `push_through` adapter for the `score` CLI.
//!
//! All work-item-mutating verbs (`aw wi`, `aw td`)
//! call `maybe_push_remote` immediately before their `commit_lifecycle`
//! (or equivalent raw `git commit`) so that the remote tracker stays in
//! lock-step with the lifecycle working copy. The helper is a no-op for the local
//! backend and for repos with no `.aw/config.toml`.

use crate::issues::{
    make_backend, push_through, resolve_default_backend, IssueBackend, LocalBackend,
};
use anyhow::Result;
use std::path::Path;

// Push the merged Issue working-copy file through the configured remote
// backend (if any). Legacy callers may still pass `.aw/issues/...`; when that
// path no longer exists, resolve the active temp-backed local issue store.
///
// Behaviour:
// - No `.aw/config.toml` → no-op (used by test fixtures).
// - `kind == "local"` → no-op (the lifecycle issue file is the storage).
// - Otherwise → `push_through(issue_path, backend, slug)`.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/remote_push.md#source
pub async fn maybe_push_remote(project_root: &Path, issue_path: &Path, slug: &str) -> Result<()> {
    let (kind, repo, host) = match resolve_default_backend(project_root) {
        Ok(t) => t,
        Err(_) => return Ok(()),
    };
    if kind == "local" {
        return Ok(());
    }
    let backend = make_backend(&kind, project_root, repo, host)?;
    let resolved_issue_path = if issue_path.exists() {
        issue_path.to_path_buf()
    } else {
        let local = LocalBackend::from_project_root(project_root);
        match local.get(slug).await? {
            Some(issue) => local.issue_path(&issue),
            None => issue_path.to_path_buf(),
        }
    };
    push_through(&resolved_issue_path, backend.as_ref(), slug).await?;
    Ok(())
}

// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/remote_push.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```
