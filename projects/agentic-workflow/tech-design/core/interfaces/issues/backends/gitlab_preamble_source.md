---
id: sdd-interfaces-issues-backends-gitlab-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# GitLab Backend Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/backends/gitlab.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitLabBackend` | projects/agentic-workflow/src/issues/backends/gitlab.rs | struct | pub | 28 |  |
| `new` | projects/agentic-workflow/src/issues/backends/gitlab.rs | function | pub | 38 | new(repo: Option<String>) -> Self |
| `with_host` | projects/agentic-workflow/src/issues/backends/gitlab.rs | function | pub | 45 | with_host(repo: Option<String>, host: Option<String>) -> Self |
## Source
<!-- type: source lang: rust -->
```rust
//! GitLab backend -- shells out to the `glab` CLI.
//!
//! The CRRR write contract round-trips through GitLab's native attributes
//! (title, state, description, labels) — see `crate::issues::labels` for
//! the label-prefix scheme that encodes the rest of `Issue`'s CRRR state.
//! `slug:*` labels are treated as legacy aliases; the GitLab issue iid is
//! the canonical identity.
//!
//! Authentication is delegated to `glab auth login`. Self-hosted hosts go
//! through the `GITLAB_HOST` environment variable.

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R6

use crate::issues::backend::IssueBackend;
use crate::issues::labels;
use crate::issues::types::{Issue, IssueFilter, IssuePatch, IssueState, IssueType};
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use serde_json::Value;
use std::process::Command;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/backends/gitlab.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "<handwrite-gap:gitlab-backend-preamble>"
    description: "Source template owns the GitLab backend module docs and imports."
```
