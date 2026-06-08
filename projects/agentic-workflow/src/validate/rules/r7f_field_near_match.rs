// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7f_field_near_match-rs.md#source
// CODEGEN-BEGIN
//! R7f — flag near-match (Levenshtein distance ≤ 2) property keys across
//! definitions sharing the same `name:`. Catches typos like `userId` vs
//! `user_id`.
//!
//! Wraps `spec_alignment::logical_rules::check` and surfaces the
//! `DefinitionConflictFieldName` / `NestedSchemaConflictFieldName` variants.

use crate::spec_alignment::models::ViolationKind;
use crate::spec_alignment::{logical_rules, parser};
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use std::path::Path;

#[derive(Debug, Default, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7f_field_near_match-rs.md#source
pub struct FieldNearMatchRule {}

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7f_field_near_match-rs.md#source
impl Rule for FieldNearMatchRule {
    fn id(&self) -> RuleId {
        RuleId::FieldNearMatch
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        let path_str = spec_path.to_string_lossy();
        let doc = parser::parse(&path_str, content);
        let violations = logical_rules::check(&doc);
        for v in violations {
            if !matches!(
                v.kind,
                ViolationKind::DefinitionConflictFieldName
                    | ViolationKind::NestedSchemaConflictFieldName
            ) {
                continue;
            }
            report.push(Finding::error(
                RuleId::FieldNearMatch,
                spec_path,
                v.message.clone(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        FieldNearMatchRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn distinct_keys_pass() {
        let spec = r#"## A
<!-- type: schema lang: json -->

```json
{"name": "Foo", "properties": {"alpha": {"type": "string"}}}
```

## B
<!-- type: schema lang: json -->

```json
{"name": "Foo", "properties": {"omega": {"type": "string"}}}
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn near_match_flagged() {
        let spec = r#"## A
<!-- type: schema lang: json -->

```json
{"name": "Foo", "properties": {"userId": {"type": "string"}}}
```

## B
<!-- type: schema lang: json -->

```json
{"name": "Foo", "properties": {"user_id": {"type": "string"}}}
```
"#;
        let r = run(spec);
        assert!(!r.is_empty());
        assert!(r.findings.iter().any(|f| f.rule == RuleId::FieldNearMatch));
    }
}

// CODEGEN-END
