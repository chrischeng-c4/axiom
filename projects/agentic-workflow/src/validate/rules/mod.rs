// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-mod-rs.md#source
// CODEGEN-BEGIN
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
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-mod-rs.md#source
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

// CODEGEN-END
