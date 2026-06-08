---
id: projects-sdd-src-issues-backends-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Standardized projects/agentic-workflow/src/issues/backends/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/backends/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `github` | projects/agentic-workflow/src/issues/backends/mod.rs | module | pub | 5 |  |
| `gitlab` | projects/agentic-workflow/src/issues/backends/mod.rs | module | pub | 6 |  |
| `local` | projects/agentic-workflow/src/issues/backends/mod.rs | module | pub | 7 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/issues/backends/mod.rs -->
```rust
//! Issue storage backend implementations.

pub mod github;
pub mod gitlab;
pub mod local;

pub use github::GitHubBackend;
pub use gitlab::GitLabBackend;
pub use local::LocalBackend;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/backends/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the issue backend facade module.
```
