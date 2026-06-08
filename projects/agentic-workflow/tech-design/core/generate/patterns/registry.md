---
id: projects-sdd-src-generate-patterns-registry-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/patterns/registry.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/patterns/registry.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `PATTERN_REGISTRY` | projects/agentic-workflow/src/generate/patterns/registry.rs | constant | pub | 20 |  |
| `pattern_registry` | projects/agentic-workflow/src/generate/patterns/registry.rs | function | pub | 14 | pattern_registry() -> &'static [UxPattern] |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/patterns/registry.rs -->
```rust
//! Built-in UX pattern registry.
//!
//! Adding a new pattern requires a code change — same principle as the
//! design system registry in tech_stack.
//!
//! Pattern definitions will be added in a future change.

use super::UxPattern;

/// Built-in pattern registry. Currently empty — patterns added incrementally.
/// @spec projects/agentic-workflow/tech-design/core/generate/patterns/registry.md#source
pub fn pattern_registry() -> &'static [UxPattern] {
    static REGISTRY: &[UxPattern] = &[];
    REGISTRY
}

/// Alias for backward compatibility
pub const PATTERN_REGISTRY: &[UxPattern] = &[];
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/patterns/registry.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete built-in UX pattern registry facade.
```
