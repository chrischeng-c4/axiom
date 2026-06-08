---
id: projects-sdd-src-test-util-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/test_util.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/test_util.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `setup_change_with_issue` | projects/agentic-workflow/src/test_util.rs | function | pub | 44 | setup_change_with_issue(slug: &str) -> (TempDir, PathBuf) |
| `setup_project_with_issue` | projects/agentic-workflow/src/test_util.rs | function | pub | 56 | setup_project_with_issue(slug: &str) -> TempDir |
| `write_minimal_issue` | projects/agentic-workflow/src/test_util.rs | function | pub | 19 | write_minimal_issue(project_root: &std::path::Path, slug: &str) |
## Source
<!-- type: source lang: rust -->

````rust
//! Test fixtures shared across SDD unit tests.
//!
//! @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R4
//! R4: StateManager::save() requires an issue file to back sync_to_issue().
//! Tests that exercise save() must create a minimal issue fixture; these
//! helpers centralise that setup so tests stay uniform across modules.

#![cfg(test)]

use std::path::PathBuf;
use tempfile::TempDir;

/// Create a minimal temp issue working copy for `<slug>` so
/// `StateManager::sync_to_issue()` can write to it. The file uses the minimum
/// frontmatter + body the structured-issue gate accepts.
/// @spec projects/agentic-workflow/tech-design/core/validate/test_util.md#source
pub(crate) fn write_minimal_issue(project_root: &std::path::Path, slug: &str) {
    let issues_dir = crate::shared::workspace::issues_path(project_root).join("open");
    std::fs::create_dir_all(&issues_dir).unwrap();
    let body = format!(
        "---\n\
         type: refactor\n\
         title: 'test({}): fixture'\n\
         state: open\n\
         ---\n\n\
         ## Problem\n\n\
         Test fixture for {}.\n\n\
         ## Requirements\n\n\
         - R1: fixture requirement\n\n\
         ## Scope\n\n\
         In scope: fixture.\n",
        slug, slug
    );
    std::fs::write(issues_dir.join(format!("{}.md", slug)), body).unwrap();
}

/// Create a temp project root with `.aw/changes/<slug>/` change dir AND a
/// backing temp issue file. Returns `(TempDir,
/// change_dir)`; hold the TempDir for lifetime.
#[allow(dead_code)]
/// @spec projects/agentic-workflow/tech-design/core/validate/test_util.md#source
pub(crate) fn setup_change_with_issue(slug: &str) -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let change_dir = tmp.path().join(".aw/changes").join(slug);
    std::fs::create_dir_all(&change_dir).unwrap();
    write_minimal_issue(tmp.path(), slug);
    (tmp, change_dir)
}

/// Create a temp project root with `.aw/changes/` dir AND a backing issue
/// file for `slug`. Returns the TempDir only — tests that manage their own
/// change_dir layout should use this.
/// @spec projects/agentic-workflow/tech-design/core/validate/test_util.md#source
pub(crate) fn setup_project_with_issue(slug: &str) -> TempDir {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".aw/changes")).unwrap();
    write_minimal_issue(tmp.path(), slug);
    tmp
}

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/test_util.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the test utility module directly from the source template.
      This helper is small and cross-language-neutral enough that a narrower
      section type would add noise instead of reusable generator capability.
```
