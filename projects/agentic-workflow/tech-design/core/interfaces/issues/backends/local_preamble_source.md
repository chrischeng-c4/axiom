---
id: sdd-interfaces-issues-backends-local-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Local Backend Preamble Source

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
```rust
//! Local filesystem backend — reads and writes `{issues_dir}/{open,closed}/*.md`.
//!
//! Issues are physically separated into `open/` and `closed/` subdirectories,
//! mirroring GitHub/GitLab's two-state model. Each issue is a Markdown file
//! with YAML frontmatter. Project-root instances store lifecycle working
//! copies under `/tmp/aw/workspaces/<workspace>/issues`; remote read-through
//! cache instances live under `/tmp/aw/issues`. Tracker-backed issues use the
//! tracker-local number (`github_id` / `gitlab_id`) as their canonical file
//! key; legacy title slugs remain readable as aliases when they already exist
//! on disk.

use crate::issues::backend::IssueBackend;
use crate::issues::types::{Issue, IssueFilter, IssuePatch, IssueState, IssueType};
use crate::parser::frontmatter::parse_document;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/backends/local.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
    description: "Source template owns the local backend module docs and imports."
```
