---
id: projects-sdd-src-ui-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Standardized projects/agentic-workflow/src/ui/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/ui/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `colors` | projects/agentic-workflow/src/ui/mod.rs | module | pub | 3 |  |
| `progress` | projects/agentic-workflow/src/ui/mod.rs | module | pub | 4 |  |
| `tables` | projects/agentic-workflow/src/ui/mod.rs | module | pub | 5 |  |
| `viewer` | projects/agentic-workflow/src/ui/mod.rs | module | pub | 8 |  |
## Source
<!-- type: source lang: rust -->

```rust
pub mod colors;
pub mod progress;
pub mod tables;

#[cfg(feature = "ui")]
pub mod viewer;

pub use colors::ColorScheme;
pub use progress::ProgressBar;
pub use tables::Table;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/ui/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the module declarations and public re-exports directly from
      the source section.
```
