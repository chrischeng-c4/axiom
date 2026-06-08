// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r3d_lowercase_enum-rs.md#source
// CODEGEN-BEGIN
//! R3d — enum `rust_type` MUST be PascalCase.
//!
//! Rust enum idiom is PascalCase (`StatePhase`, `ReportKind`). Lowercase
//! values (`statephase`, `report_kind`) produce awkward generated code that
//! either fails clippy's `enum_variant_names` / naming lints or compiles
//! into something non-idiomatic.
//!
//! Scope: only `rust_type` values attached to a schema whose `type: enum`
//! (or whose properties carry `enum: [...]` — the openapi shape). Struct
//! rust_types (u16, String, Vec<T>) are unaffected.

use crate::validate::rules::r3b_nullable_required::extract_yaml_blocks;
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use serde_yaml::Value;
use std::path::Path;

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r3d_lowercase_enum-rs.md#source
impl Rule for LowercaseEnumRule {
    fn id(&self) -> RuleId {
        RuleId::LowercaseEnum
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        for block in extract_yaml_blocks(content) {
            let Ok(yaml) = serde_yaml::from_str::<Value>(&block.body) else {
                continue;
            };
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

fn check_schema(schema: &Value, spec_path: &Path, report: &mut RuleReport) {
    let Some(map) = schema.as_mapping() else {
        return;
    };

    // Case 1: top-level enum schema (`type: enum` / `enum: [...]` with rust_type).
    let is_enum = map
        .get(Value::String("type".into()))
        .and_then(|v| v.as_str())
        .map(|t| t == "enum" || t == "string")
        .unwrap_or(false)
        && map.contains_key(Value::String("enum".into()));
    if is_enum {
        if let Some(rt) = map
            .get(Value::String("rust_type".into()))
            .and_then(|v| v.as_str())
        {
            flag_if_not_pascal(rt, spec_path, report, "rust_type");
        }
    }

    // Case 2: per-property enum with rust_type (OpenAPI shape).
    if let Some(props) = map
        .get(Value::String("properties".into()))
        .and_then(|v| v.as_mapping())
    {
        for (k, v) in props {
            let Some(prop_name) = k.as_str() else {
                continue;
            };
            let Some(pm) = v.as_mapping() else { continue };
            if !pm.contains_key(Value::String("enum".into())) {
                continue;
            }
            if let Some(rt) = pm
                .get(Value::String("rust_type".into()))
                .and_then(|v| v.as_str())
            {
                flag_if_not_pascal(
                    rt,
                    spec_path,
                    report,
                    &format!("properties.{}.rust_type", prop_name),
                );
            }
        }
    }
}

fn flag_if_not_pascal(rt: &str, spec_path: &Path, report: &mut RuleReport, path_hint: &str) {
    if is_pascal_case(rt) {
        return;
    }
    report.push(
        Finding::error(
            RuleId::LowercaseEnum,
            spec_path,
            format!(
                "enum rust_type `{}` is not PascalCase (expected e.g. `StatePhase`)",
                rt
            ),
        )
        .with_path(path_hint.to_string()),
    );
}

/// PascalCase = starts with uppercase ASCII letter; contains only letters
/// + digits (no `_`, no `-`). Permits nested generics (`Foo<Bar>`) since
/// the outer name is what matters for an enum alias.
fn is_pascal_case(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Strip generic suffix: `Foo<Bar>` → `Foo`.
    let head = trimmed.split('<').next().unwrap_or(trimmed).trim();
    let mut chars = head.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() => {}
        _ => return false,
    }
    head.chars().all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        LowercaseEnumRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn pascal_enum_clean() {
        let spec = r#"
```yaml
schemas:
  - type: enum
    enum: [foo, bar]
    rust_type: StatePhase
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn lowercase_enum_flagged() {
        let spec = r#"
```yaml
schemas:
  - type: enum
    enum: [foo, bar]
    rust_type: statephase
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].message.contains("statephase"));
    }

    #[test]
    fn snake_case_enum_flagged() {
        let spec = r#"
```yaml
schemas:
  - type: enum
    enum: [foo, bar]
    rust_type: state_phase
```
"#;
        assert_eq!(run(spec).findings.len(), 1);
    }

    #[test]
    fn per_property_lowercase_enum_flagged() {
        let spec = r#"
```yaml
schemas:
  - properties:
      status:
        enum: [open, closed]
        rust_type: statusphase
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].path.as_deref() == Some("properties.status.rust_type"));
    }

    #[test]
    fn non_enum_schema_ignored() {
        let spec = r#"
```yaml
schemas:
  - properties:
      status:
        rust_type: u16
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn struct_rust_type_ignored() {
        // u16 / String are not PascalCase by the strict rule but they are not
        // enum rust_types either — this rule stays scoped to enum schemas.
        let spec = r#"
```yaml
schemas:
  - type: object
    rust_type: Foo
    properties:
      x:
        rust_type: u16
```
"#;
        assert!(run(spec).is_empty());
    }
}
/// LowercaseEnumRule validation rule (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/validate/rules/r3d_lowercase_enum.md#schema
pub struct LowercaseEnumRule {}

// CODEGEN-END
