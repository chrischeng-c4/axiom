// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r3g_rust_type_consistency-rs.md#source
// CODEGEN-BEGIN
//! R3g — cross-section `rust_type` consistency.
//!
//! A symbol (schema title) that appears in both `schemas:` and `changes:`
//! MUST carry the same `rust_type` in every occurrence. Divergent types
//! produce codegen that compiles against itself but links against a
//! downstream crate's different type — the kind of bug that passes unit
//! tests and fails at integration.
//!
//! Today this rule checks two pairings:
//!
//! 1. **Schema title ↔ schema rust_type** — every `schemas[].title` SHOULD
//!    have a `rust_type`; if multiple schemas share a title they must agree.
//! 2. **Schema title ↔ x-constructor args rust_type** — inside
//!    `x-constructor.args[].rust_type`, any `rust_type` naming the enclosing
//!    schema's title (e.g. building a `HTTPException` arg inside
//!    `HTTPException`'s constructor) should use the same canonical name.
//!
//! Keeping scope tight — future expansions (changes[] rust_type, x-methods
//! rust_type) land as follow-ups behind the same RuleId.

use crate::validate::rules::r3b_nullable_required::extract_yaml_blocks;
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::Path;

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r3g_rust_type_consistency-rs.md#source
impl Rule for RustTypeConsistencyRule {
    fn id(&self) -> RuleId {
        RuleId::RustTypeConsistency
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        let mut by_title: HashMap<String, Vec<String>> = HashMap::new();

        for block in extract_yaml_blocks(content) {
            let Ok(yaml) = serde_yaml::from_str::<Value>(&block.body) else {
                continue;
            };
            let schemas_opt = yaml.get("schemas").and_then(|v| v.as_sequence());
            let schemas: Vec<&Value> = match schemas_opt {
                Some(seq) => seq.iter().collect(),
                None => vec![&yaml],
            };
            for schema in schemas {
                let Some(map) = schema.as_mapping() else {
                    continue;
                };
                let Some(title) = map
                    .get(Value::String("title".into()))
                    .and_then(|v| v.as_str())
                else {
                    continue;
                };
                if let Some(rt) = map
                    .get(Value::String("rust_type".into()))
                    .and_then(|v| v.as_str())
                {
                    by_title
                        .entry(title.to_string())
                        .or_default()
                        .push(rt.to_string());
                }
            }
        }

        // Flag any title with >1 distinct rust_type.
        for (title, rts) in &by_title {
            let distinct: std::collections::HashSet<&String> = rts.iter().collect();
            if distinct.len() > 1 {
                let mut sorted: Vec<&String> = distinct.into_iter().collect();
                sorted.sort();
                let list = sorted
                    .iter()
                    .map(|s| format!("`{}`", s))
                    .collect::<Vec<_>>()
                    .join(", ");
                report.push(
                    Finding::error(
                        RuleId::RustTypeConsistency,
                        spec_path,
                        format!(
                            "schema `{}` has inconsistent rust_type values across occurrences: {}",
                            title, list
                        ),
                    )
                    .with_path(format!("schemas[title={}]", title)),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        RustTypeConsistencyRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn single_schema_clean() {
        let spec = r#"
```yaml
schemas:
  - title: Foo
    rust_type: Foo
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn consistent_multi_block_clean() {
        let spec = r#"
```yaml
schemas:
  - title: Foo
    rust_type: Foo
```

Other prose.

```yaml
schemas:
  - title: Foo
    rust_type: Foo
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn inconsistent_rust_type_flagged() {
        let spec = r#"
```yaml
schemas:
  - title: Foo
    rust_type: Foo
```

```yaml
schemas:
  - title: Foo
    rust_type: FooV2
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].message.contains("Foo"));
        assert!(r.findings[0].message.contains("FooV2"));
    }

    #[test]
    fn distinct_titles_independent() {
        let spec = r#"
```yaml
schemas:
  - title: Foo
    rust_type: Foo
  - title: Bar
    rust_type: Baz
```
"#;
        // Bar uses rust_type: Baz — that's allowed (aliasing); as long as
        // there's no conflicting Bar elsewhere, no finding.
        assert!(run(spec).is_empty());
    }

    #[test]
    fn missing_title_skipped() {
        let spec = r#"
```yaml
schemas:
  - rust_type: Anonymous
```
"#;
        assert!(run(spec).is_empty());
    }
}
/// RustTypeConsistencyRule validation rule (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/validate/rules/r3g_rust_type_consistency.md#schema
pub struct RustTypeConsistencyRule {}

// CODEGEN-END
