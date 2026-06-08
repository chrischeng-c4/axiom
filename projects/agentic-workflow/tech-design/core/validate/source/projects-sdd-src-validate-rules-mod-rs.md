---
id: projects-sdd-src-validate-rules-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/rules/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/rules/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `all_rules` | projects/agentic-workflow/src/validate/rules/mod.rs | function | pub | 48 | all_rules() -> Vec<Box<dyn Rule>> |
| `r3a_double_option` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 9 |  |
| `r3b_nullable_required` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 10 |  |
| `r3c_orphan_binding` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 11 |  |
| `r3d_lowercase_enum` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 12 |  |
| `r3e_impl_mode_misuse` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 13 |  |
| `r3f_codegen_ready` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 14 |  |
| `r3g_rust_type_consistency` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 15 |  |
| `r6a_loose_root_file` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 16 |  |
| `r6b_unexpected_subdir` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 17 |  |
| `r7a_missing_section_annotation` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 18 |  |
| `r7b_format_priority_violation` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 19 |  |
| `r7c_duplicate_section` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 20 |  |
| `r7d_orphan_requirement` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 21 |  |
| `r7e_schema_conflict` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 22 |  |
| `r7f_field_near_match` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 23 |  |
| `section_format` | projects/agentic-workflow/src/validate/rules/mod.rs | module | pub | 24 |  |
## Source
<!-- type: source lang: rust -->

````rust
//! Rule implementations.
//!
//! Each `r{N}{x}_<name>.rs` file implements one `Rule` trait impl. Adding a
//! new rule: create the module, `pub use` the struct here, and add it to
//! `all_rules()`.

pub mod r3a_double_option;
pub mod r3b_nullable_required;
pub mod r3c_orphan_binding;
pub mod r3d_lowercase_enum;
pub mod r3e_impl_mode_misuse;
pub mod r3f_codegen_ready;
pub mod r3g_rust_type_consistency;
pub mod r6a_loose_root_file;
pub mod r6b_unexpected_subdir;
pub mod r7a_missing_section_annotation;
pub mod r7b_format_priority_violation;
pub mod r7c_duplicate_section;
pub mod r7d_orphan_requirement;
pub mod r7e_schema_conflict;
pub mod r7f_field_near_match;
pub mod section_format;

pub use r3a_double_option::DoubleOptionRule;
pub use r3b_nullable_required::NullableRequiredRule;
pub use r3c_orphan_binding::OrphanBindingRule;
pub use r3d_lowercase_enum::LowercaseEnumRule;
pub use r3e_impl_mode_misuse::ImplModeMisuseRule;
pub use r3f_codegen_ready::CodegenReadyRule;
pub use r3g_rust_type_consistency::RustTypeConsistencyRule;
pub use r6a_loose_root_file::LooseRootFileRule;
pub use r6b_unexpected_subdir::UnexpectedSubdirRule;
pub use r7a_missing_section_annotation::MissingSectionAnnotationRule;
pub use r7b_format_priority_violation::FormatPriorityViolationRule;
pub use r7c_duplicate_section::DuplicateSectionRule;
pub use r7d_orphan_requirement::OrphanRequirementRule;
pub use r7e_schema_conflict::SchemaConflictRule;
pub use r7f_field_near_match::FieldNearMatchRule;
pub use section_format::SectionFormatRule;

use crate::validate::Rule;

/// Full rule registry. Returns the default set (all implemented rules).
/// Callers run this list against every spec in scope.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(DoubleOptionRule {}),
        Box::new(NullableRequiredRule {}),
        Box::new(OrphanBindingRule {}),
        Box::new(LowercaseEnumRule {}),
        Box::new(ImplModeMisuseRule {}),
        Box::new(CodegenReadyRule {}),
        Box::new(RustTypeConsistencyRule {}),
        Box::new(SectionFormatRule::default()),
        Box::new(LooseRootFileRule::default()),
        Box::new(UnexpectedSubdirRule::default()),
        Box::new(MissingSectionAnnotationRule::default()),
        Box::new(FormatPriorityViolationRule::default()),
        Box::new(DuplicateSectionRule::default()),
        Box::new(OrphanRequirementRule::default()),
        Box::new(SchemaConflictRule::default()),
        Box::new(FieldNearMatchRule::default()),
    ]
}
````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
