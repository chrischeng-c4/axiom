---
id: projects-sdd-src-generate-gen-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/gen/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `operations` | projects/agentic-workflow/src/generate/gen/mod.rs | module | pub | 9 |  |
| `python` | projects/agentic-workflow/src/generate/gen/mod.rs | module | pub | 10 |  |
| `rust` | projects/agentic-workflow/src/generate/gen/mod.rs | module | pub | 11 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/mod.rs -->
```rust
//! Code generators for all target languages.
//!
//! Currently implements: Rust (structural + behavioral).
//! Python and TypeScript translators share the same `AbstractType` enum
//! but have deferred implementations.

pub mod rust;

pub use rust::*;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: Source template owns the complete language generator module aggregator.
```
