---
id: sdd-interfaces-issues-backends-github-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# GitHub Backend Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/backends/github.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitHubBackend` | projects/agentic-workflow/src/issues/backends/github.rs | struct | pub | 27 |  |
| `new` | projects/agentic-workflow/src/issues/backends/github.rs | function | pub | 37 | new(repo: Option<String>) -> Self |
| `with_host` | projects/agentic-workflow/src/issues/backends/github.rs | function | pub | 44 | with_host(repo: Option<String>, host: Option<String>) -> Self |
## Source
<!-- type: source lang: rust -->

```rust
//! GitHub backend — shells out to the `gh` CLI.
//!
//! The CRRR write contract round-trips through GitHub's native attributes
//! (title, state, body, labels) — see `crate::issues::labels` for the
//! label-prefix scheme that encodes `phase`, `review_count`, `flagged_sections`,
//! `fill_retry_count`, `ship_status`, and `ship_commit` as labels on the
//! GitHub issue. `slug:*` labels are treated as legacy aliases; the GitHub
//! issue number is the canonical identity.
//!
//! Authentication is delegated to the `gh` CLI (user must have run
//! `gh auth login` beforehand).

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
  - path: projects/agentic-workflow/src/issues/backends/github.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "<handwrite-gap:github-backend-preamble>"
    description: "Source template owns the GitHub backend module docs and imports."
```
