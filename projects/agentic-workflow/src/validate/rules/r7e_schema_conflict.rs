// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7e_schema_conflict-rs.md#source
// CODEGEN-BEGIN
//! R7e — flag conflicting type/enum/format on the same property key when the
//! same named definition (`name:` field) appears in multiple JSON code
//! blocks within a spec.
//!
//! Wraps `spec_alignment::logical_rules::check` and surfaces only the
//! `DefinitionConflictSchema` and `NestedSchemaConflictSchema` variants.

use crate::spec_alignment::models::ViolationKind;
use crate::spec_alignment::{logical_rules, parser};
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use std::path::Path;

#[derive(Debug, Default, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7e_schema_conflict-rs.md#source
pub struct SchemaConflictRule {}

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7e_schema_conflict-rs.md#source
impl Rule for SchemaConflictRule {
    fn id(&self) -> RuleId {
        RuleId::SchemaConflict
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        let path_str = spec_path.to_string_lossy();
        let doc = parser::parse(&path_str, content);
        let violations = logical_rules::check(&doc);
        for v in violations {
            if !matches!(
                v.kind,
                ViolationKind::DefinitionConflictSchema | ViolationKind::NestedSchemaConflictSchema
            ) {
                continue;
            }
            let mut f = Finding::error(RuleId::SchemaConflict, spec_path, v.message.clone());
            if let Some(field) = v.field {
                f = f.with_path(field);
            }
            report.push(f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        SchemaConflictRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn matching_schemas_pass() {
        let spec = r#"## A
<!-- type: schema lang: json -->

```json
{"name": "Foo", "properties": {"x": {"type": "string"}}}
```

## B
<!-- type: schema lang: json -->

```json
{"name": "Foo", "properties": {"x": {"type": "string"}}}
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn conflicting_field_type_flagged() {
        let spec = r#"## A
<!-- type: schema lang: json -->

```json
{"name": "Foo", "properties": {"x": {"type": "string"}}}
```

## B
<!-- type: schema lang: json -->

```json
{"name": "Foo", "properties": {"x": {"type": "integer"}}}
```
"#;
        let r = run(spec);
        assert!(!r.is_empty());
        assert!(r.findings.iter().any(|f| f.rule == RuleId::SchemaConflict));
    }
}

// CODEGEN-END
