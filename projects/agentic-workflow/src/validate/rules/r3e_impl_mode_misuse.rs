// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r3e_impl_mode_misuse-rs.md#source
// CODEGEN-BEGIN
//! R3e — `impl_mode` misuse.
//!
//! Two violations caught here:
//!
//! 1. **Missing or invalid value.** `impl_mode` MUST be present and must be
//!    one of `codegen` or `hand-written`. Any other string is rejected.
//!
//! 2. **Mixed impl_mode inside a single entry is impossible by construction**
//!    (one `impl_mode` per `changes[]` entry) — but the Rule 1/2-1/2-2
//!    policy also forbids using `impl_mode` on sections where it makes no
//!    sense. Currently the only entry-level field is `changes[].impl_mode`;
//!    rule is forward-compatible with tightening later.
//!
//! Scope: `changes:` list entries. One finding per offending entry.

use crate::validate::rules::r3b_nullable_required::extract_yaml_blocks;
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use serde_yaml::Value;
use std::path::Path;

const ALLOWED: &[&str] = &["codegen", "hand-written"];

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r3e_impl_mode_misuse-rs.md#source
impl Rule for ImplModeMisuseRule {
    fn id(&self) -> RuleId {
        RuleId::ImplModeMisuse
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        for block in extract_yaml_blocks(content) {
            let Ok(yaml) = serde_yaml::from_str::<Value>(&block.body) else {
                continue;
            };
            let changes = yaml
                .get("changes")
                .and_then(|v| v.as_sequence())
                .or_else(|| yaml.as_sequence());
            let Some(changes) = changes else { continue };
            for (idx, entry) in changes.iter().enumerate() {
                let Some(map) = entry.as_mapping() else {
                    continue;
                };
                let path_hint = map
                    .get(Value::String("path".into()))
                    .and_then(|v| v.as_str())
                    .map(|s| format!("changes[{}] ({})", idx, s))
                    .unwrap_or_else(|| format!("changes[{}]", idx));
                let Some(val) = map
                    .get(Value::String("impl_mode".into()))
                    .and_then(|v| v.as_str())
                else {
                    report.push(
                        Finding::error(
                            RuleId::ImplModeMisuse,
                            spec_path,
                            "missing impl_mode; every changes[] entry must declare codegen or hand-written",
                        )
                        .with_path(path_hint),
                    );
                    continue;
                };
                if !ALLOWED.contains(&val) {
                    report.push(
                        Finding::error(
                            RuleId::ImplModeMisuse,
                            spec_path,
                            format!("impl_mode `{}` is not in {{codegen, hand-written}}", val),
                        )
                        .with_path(path_hint),
                    );
                }
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
        ImplModeMisuseRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn codegen_is_allowed() {
        let spec = r#"
```yaml
changes:
  - path: foo.rs
    action: create
    impl_mode: codegen
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn hand_written_is_allowed() {
        let spec = r#"
```yaml
changes:
  - path: foo.rs
    action: create
    impl_mode: hand-written
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn missing_impl_mode_is_flagged() {
        let spec = r#"
```yaml
changes:
  - path: foo.rs
    action: create
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].message.contains("missing impl_mode"));
    }

    #[test]
    fn arbitrary_value_is_flagged() {
        let spec = r#"
```yaml
changes:
  - path: foo.rs
    impl_mode: handwritten
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].message.contains("handwritten"));
    }

    #[test]
    fn typo_codegen_flagged() {
        let spec = r#"
```yaml
changes:
  - path: foo.rs
    impl_mode: codegend
```
"#;
        assert_eq!(run(spec).findings.len(), 1);
    }

    #[test]
    fn multiple_invalid_entries_all_flagged() {
        let spec = r#"
```yaml
changes:
  - path: a.rs
    impl_mode: auto
  - path: b.rs
    impl_mode: manual
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 2);
    }

    #[test]
    fn path_hint_includes_entry_index() {
        let spec = r#"
```yaml
changes:
  - path: a.rs
    impl_mode: codegen
  - path: b.rs
    impl_mode: wrong
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0]
            .path
            .as_deref()
            .unwrap()
            .contains("changes[1]"));
        assert!(r.findings[0].path.as_deref().unwrap().contains("b.rs"));
    }
}
/// ImplModeMisuseRule validation rule (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/validate/rules/r3e_impl_mode_misuse.md#schema
pub struct ImplModeMisuseRule {}

// CODEGEN-END
