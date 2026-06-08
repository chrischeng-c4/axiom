---
id: sdd-generate-engine-error-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Template Error Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/engine/error.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TemplateError` | projects/agentic-workflow/src/generate/engine/error.rs | enum | pub | 12 |  |
## Source
<!-- type: source lang: rust -->

```rust
//! Template engine error types

use std::path::PathBuf;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/engine/error.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
    description: "Source template owns template error module docs and imports."
```
