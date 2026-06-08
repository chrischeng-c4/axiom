---
id: projects-sdd-src-validate-rules-r3c_orphan_binding-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/rules/r3c_orphan_binding.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/rules/r3c_orphan_binding.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `OrphanBindingRule` | projects/agentic-workflow/src/validate/rules/r3c_orphan_binding.rs | struct | pub | 202 |  |
## Source
<!-- type: source lang: rust -->

````rust
//! R3c — `x-mamba-binding` must not be an orphan.
//!
//! An `x-mamba-binding` attaches a Mamba symbol (`symbol`, `extern_fn`,
//! `signature`) to a schema. Two common orphan shapes to reject:
//!
//! 1. **Symbol mismatch** — binding `symbol` differs from the enclosing
//!    schema's `title`. The generator emits the FFI shim named after the
//!    `title`; a mismatched symbol produces a binding that never resolves.
//! 2. **Incomplete binding** — required sub-keys (`symbol`, `extern_fn`,
//!    `signature`) missing or empty. The downstream generator requires all
//!    three; a partial binding is a silent codegen failure.

use crate::validate::rules::r3b_nullable_required::extract_yaml_blocks;
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use serde_yaml::Value;
use std::path::Path;

impl Rule for OrphanBindingRule {
    fn id(&self) -> RuleId {
        RuleId::OrphanBinding
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
    let Some(binding) = map
        .get(Value::String("x-mamba-binding".into()))
        .and_then(|v| v.as_mapping())
    else {
        return;
    };

    // Required sub-keys.
    for key in ["symbol", "extern_fn", "signature"] {
        let val = binding
            .get(Value::String(key.into()))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if val.trim().is_empty() {
            report.push(
                Finding::error(
                    RuleId::OrphanBinding,
                    spec_path,
                    format!("x-mamba-binding missing or empty field `{}`", key),
                )
                .with_path(format!("x-mamba-binding.{}", key)),
            );
        }
    }

    // Symbol must match enclosing schema title (when title is present).
    let title = map
        .get(Value::String("title".into()))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let symbol = binding
        .get(Value::String("symbol".into()))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !title.is_empty() && !symbol.is_empty() && title != symbol {
        report.push(
            Finding::error(
                RuleId::OrphanBinding,
                spec_path,
                format!(
                    "x-mamba-binding.symbol `{}` does not match enclosing schema title `{}`",
                    symbol, title
                ),
            )
            .with_path("x-mamba-binding.symbol"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        OrphanBindingRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn complete_matching_binding_is_clean() {
        let spec = r#"
```yaml
schemas:
  - title: HTTPException
    x-mamba-binding:
      symbol: HTTPException
      extern_fn: http_exception_new
      signature: "HTTPException(status_code: int)"
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn no_binding_is_clean() {
        let spec = r#"
```yaml
schemas:
  - title: Foo
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn missing_extern_fn_flagged() {
        let spec = r#"
```yaml
schemas:
  - title: Foo
    x-mamba-binding:
      symbol: Foo
      signature: "Foo()"
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].message.contains("extern_fn"));
    }

    #[test]
    fn empty_signature_flagged() {
        let spec = r#"
```yaml
schemas:
  - title: Foo
    x-mamba-binding:
      symbol: Foo
      extern_fn: foo_new
      signature: ""
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].message.contains("signature"));
    }

    #[test]
    fn symbol_mismatch_with_title_flagged() {
        let spec = r#"
```yaml
schemas:
  - title: HTTPException
    x-mamba-binding:
      symbol: HttpExn
      extern_fn: http_exception_new
      signature: "HttpExn()"
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0]
            .message
            .contains("does not match enclosing schema title"));
    }

    #[test]
    fn multiple_missing_fields_all_flagged() {
        let spec = r#"
```yaml
schemas:
  - title: Foo
    x-mamba-binding:
      symbol: ""
      extern_fn: ""
      signature: ""
```
"#;
        let r = run(spec);
        assert_eq!(r.findings.len(), 3);
    }
}
/// OrphanBindingRule validation rule (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/validate/rules/r3c_orphan_binding.md#schema
pub struct OrphanBindingRule {}
````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r3c_orphan_binding.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
