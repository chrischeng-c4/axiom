---
id: projects-sdd-src-validate-rules-r7e_schema_conflict-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/rules/r7e_schema_conflict.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/rules/r7e_schema_conflict.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SchemaConflictRule` | projects/agentic-workflow/src/validate/rules/r7e_schema_conflict.rs | struct | pub | 17 |  |
## Source
<!-- type: source lang: rust -->

````rust
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
pub struct SchemaConflictRule {}

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
````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r7e_schema_conflict.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
