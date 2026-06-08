---
id: projects-sdd-src-validate-rules-r3b_nullable_required-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/rules/r3b_nullable_required.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/rules/r3b_nullable_required.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `NullableRequiredRule` | projects/agentic-workflow/src/validate/rules/r3b_nullable_required.rs | struct | pub | 269 |  |
| `YamlBlock` | projects/agentic-workflow/src/validate/rules/r3b_nullable_required.rs | struct | pub | 126 |  |
| `extract_yaml_blocks` | projects/agentic-workflow/src/validate/rules/r3b_nullable_required.rs | function | pub | 132 | extract_yaml_blocks(content: &str) -> Vec<YamlBlock> |
## Source
<!-- type: source lang: rust -->

````rust
//! R3b — `required: true` with nullable `rust_type` is contradictory, and
//! vice versa.
//!
//! JSON Schema semantics collide with Rust `Option<T>` semantics here:
//!
//! - A property marked `required: true` MUST be present in serialised JSON.
//!   If its `rust_type` is `Option<T>`, serde will emit `"field": null` or
//!   omit it depending on `serde(skip_serializing_if = ...)` — either way
//!   the API shape diverges from the spec.
//!
//! - Conversely, a property with `rust_type: String` but NOT in the
//!   `required:` list is nullable per schema but non-nullable per Rust.
//!   Deserialising a missing field fails instead of becoming `None`.
//!
//! This rule catches both directions. It operates on YAML blocks under
//! `schemas:` / `properties:` — the two places where `required` + `rust_type`
//! can be stated per-field.
//!
//! Implementation: parse each YAML code block in the spec, walk schemas,
//! collect per-field `(required, rust_type)` pairs, emit a finding if they
//! contradict.

use crate::validate::{Finding, Rule, RuleId, RuleReport};
use serde_yaml::Value;
use std::path::Path;

impl Rule for NullableRequiredRule {
    fn id(&self) -> RuleId {
        RuleId::NullableRequired
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        for block in extract_yaml_blocks(content) {
            let Ok(yaml) = serde_yaml::from_str::<Value>(&block.body) else {
                continue;
            };
            // Top-level `schemas:` list (the TD shape used by `aw td audit`
            // / the existing generators) OR single-schema top level.
            if let Some(schemas) = yaml.get("schemas").and_then(|v| v.as_sequence()) {
                for schema in schemas {
                    check_schema(schema, spec_path, report);
                }
            } else {
                check_schema(&yaml, spec_path, report);
            }
        }
    }
}

/// Inspect one schema object for `(required, properties, rust_type)` violations.
fn check_schema(schema: &Value, spec_path: &Path, report: &mut RuleReport) {
    let Some(map) = schema.as_mapping() else {
        return;
    };
    let required: std::collections::HashSet<String> = map
        .get(Value::String("required".into()))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let Some(props) = map
        .get(Value::String("properties".into()))
        .and_then(|v| v.as_mapping())
    else {
        return;
    };
    for (k, v) in props {
        let Some(field) = k.as_str() else { continue };
        let Some(prop_map) = v.as_mapping() else {
            continue;
        };
        let rust_type = prop_map
            .get(Value::String("rust_type".into()))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if rust_type.is_empty() {
            continue;
        }
        let is_optional_type = is_option_type(rust_type);
        let is_required = required.contains(field);
        match (is_required, is_optional_type) {
            (true, true) => {
                report.push(
                    Finding::error(
                        RuleId::NullableRequired,
                        spec_path,
                        format!(
                            "field `{}` is `required: true` but rust_type `{}` is nullable",
                            field, rust_type
                        ),
                    )
                    .with_path(format!("properties.{}", field)),
                );
            }
            (false, false) => {
                report.push(
                    Finding::error(
                        RuleId::NullableRequired,
                        spec_path,
                        format!(
                            "field `{}` is NOT in `required:` but rust_type `{}` is non-nullable",
                            field, rust_type
                        ),
                    )
                    .with_path(format!("properties.{}", field)),
                );
            }
            _ => {}
        }
    }
}

fn is_option_type(rust_type: &str) -> bool {
    let t = rust_type.trim();
    t.starts_with("Option<") || t.starts_with("Option <")
}

/// Single fenced ```yaml block. `body` is the code (no fences).
pub(crate) struct YamlBlock {
    pub body: String,
}

/// Extract every ```yaml / ```yml code block body from Markdown text.
pub(crate) fn extract_yaml_blocks(content: &str) -> Vec<YamlBlock> {
    let mut blocks = Vec::new();
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```yaml") || trimmed.starts_with("```yml") {
            let mut body = String::new();
            for inner in lines.by_ref() {
                if inner.trim_start().starts_with("```") {
                    break;
                }
                body.push_str(inner);
                body.push('\n');
            }
            blocks.push(YamlBlock { body });
        }
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        NullableRequiredRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn required_non_optional_is_clean() {
        let spec = r#"
```yaml
schemas:
  - title: T
    type: object
    required: [a]
    properties:
      a:
        rust_type: u16
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn optional_not_required_is_clean() {
        let spec = r#"
```yaml
schemas:
  - properties:
      a:
        rust_type: Option<String>
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn required_but_option_is_flagged() {
        let spec = r#"
```yaml
schemas:
  - required: [a]
    properties:
      a:
        rust_type: Option<String>
```
"#;
        let report = run(spec);
        assert_eq!(report.findings.len(), 1);
        let f = &report.findings[0];
        assert_eq!(f.rule, RuleId::NullableRequired);
        assert!(f.message.contains("required: true"));
        assert!(f.message.contains("nullable"));
    }

    #[test]
    fn not_required_non_option_is_flagged() {
        let spec = r#"
```yaml
schemas:
  - required: []
    properties:
      a:
        rust_type: u16
```
"#;
        let report = run(spec);
        assert_eq!(report.findings.len(), 1);
        let f = &report.findings[0];
        assert!(f.message.contains("non-nullable"));
    }

    #[test]
    fn multiple_violations_all_reported() {
        let spec = r#"
```yaml
schemas:
  - required: [a]
    properties:
      a:
        rust_type: Option<u16>
      b:
        rust_type: String
```
"#;
        let report = run(spec);
        assert_eq!(report.findings.len(), 2);
    }

    #[test]
    fn yaml_block_without_schemas_does_not_panic() {
        let spec = r#"
```yaml
changes:
  - path: foo.rs
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn malformed_yaml_is_skipped_silently() {
        let spec = r#"
```yaml
schemas: [
```
"#;
        // Malformed YAML — rule should not panic; just skip the block.
        assert!(run(spec).is_empty());
    }
}
/// NullableRequiredRule validation rule (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/validate/rules/r3b_nullable_required.md#schema
pub struct NullableRequiredRule {}
````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r3b_nullable_required.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
