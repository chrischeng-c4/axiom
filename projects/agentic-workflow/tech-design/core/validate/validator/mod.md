---
id: projects-sdd-src-validator-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validator/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validator/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `challenge` | projects/agentic-workflow/src/validator/mod.rs | module | pub | 3 |  |
| `consistency` | projects/agentic-workflow/src/validator/mod.rs | module | pub | 4 |  |
| `fix` | projects/agentic-workflow/src/validator/mod.rs | module | pub | 5 |  |
| `format` | projects/agentic-workflow/src/validator/mod.rs | module | pub | 6 |  |
| `schema` | projects/agentic-workflow/src/validator/mod.rs | module | pub | 7 |  |
| `semantic` | projects/agentic-workflow/src/validator/mod.rs | module | pub | 8 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/validator/mod.rs -->
```rust
pub mod challenge;
pub mod consistency;
pub mod fix;
pub mod format;
pub mod schema;
pub mod semantic;

pub use challenge::ChallengeValidator;
pub use consistency::ConsistencyValidator;
pub use fix::{AutoFixer, FixResult};
pub use format::SpecFormatValidator;
pub use schema::{
    validate_frontmatter_content, validate_frontmatter_schema, DocumentType, SchemaValidator,
};
pub use semantic::SemanticValidator;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validator/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Codegen owns the module facade through a source template.
```
