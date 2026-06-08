---
id: projects-sdd-src-validate-rules-r7f_field_near_match-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/rules/r7f_field_near_match.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/rules/r7f_field_near_match.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FieldNearMatchRule` | projects/agentic-workflow/src/validate/rules/r7f_field_near_match.rs | struct | pub | 17 |  |
## Source
<!-- type: source lang: rust -->

````rust
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
pub struct FieldNearMatchRule {}

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
````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r7f_field_near_match.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
