---
id: projects-sdd-src-validate-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `router` | projects/agentic-workflow/src/validate/mod.rs | module | pub | 23 |  |
| `rule` | projects/agentic-workflow/src/validate/mod.rs | module | pub | 24 |  |
| `rules` | projects/agentic-workflow/src/validate/mod.rs | module | pub | 25 |  |
| `runner` | projects/agentic-workflow/src/validate/mod.rs | module | pub | 26 |  |
## Source
<!-- type: source lang: rust -->

````rust
//! TD spec rule checker.
//!
//! Runs authoring-lint + consistency rules against tech-design specs.
//! Called by `aw td validate <path>` where `<path>` is a slug (commit-gate),
//! a spec-space directory prefix (read-only walk), or a single spec file.
//!
//! Distinct from `crate::validator` (which validates generic spec document
//! structure — headings, scenarios, WHEN/THEN). This module validates
//! TD-specific content rules: rust_type shape, x-mamba-binding integrity,
//! impl_mode discipline, cross-section consistency.
//!
//! Rule catalog (R-ids from issue `enhancement-split-validate-spec-side-from-audit-code-side-cove`):
//! - R3a: double-Option — reject `Option<Option<T>>` in any `rust_type`
//! - R3b: nullable/required contradiction
//! - R3c: orphan x-mamba-binding
//! - R3d: lowercase enum rust_type
//! - R3e: impl_mode misuse
//! - R3f: codegen-ready gate (Mermaid Plus frontmatter; skipped for Rule 2-2)
//! - R3g: cross-section rust_type consistency

pub mod router;
pub mod rule;
pub mod rules;
pub mod runner;

pub use router::{classify, resolve_spec_files, PathShape};
pub use rule::{Finding, Rule, RuleId, RuleReport, Severity};
pub use runner::run_rules;
````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
