---
id: projects-sdd-src-validate-rules-r3a_double_option-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/rules/r3a_double_option.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/rules/r3a_double_option.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DoubleOptionRule` | projects/agentic-workflow/src/validate/rules/r3a_double_option.rs | struct | pub | 168 |  |
## Source
<!-- type: source lang: rust -->

````rust
//! R3a — reject `Option<Option<T>>` in any `rust_type` value.
//!
//! Motivation: double-`Option` is almost never what the author means. In
//! serde + JSON Schema flows it collapses to a single `Option<T>` anyway,
//! so the inner `Option` is dead weight; worse, it hides the intent
//! between "field may be absent" vs "field may hold null". Codegen targets
//! `rust_type` verbatim, so letting this slip through produces awkward
//! downstream APIs.
//!
//! Implementation: regex scan over the raw spec text. The `rust_type:` key
//! appears in both `schemas:` YAML blocks and `x-constructor.args[].rust_type`.
//! A full YAML parse isn't worth it for this check — the offending pattern
//! is unambiguous in text.

use crate::validate::{Finding, Rule, RuleId, RuleReport};
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

impl Rule for DoubleOptionRule {
    fn id(&self) -> RuleId {
        RuleId::DoubleOption
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        for (idx, line) in content.lines().enumerate() {
            let Some(rust_type) = extract_rust_type_value(line) else {
                continue;
            };
            if contains_double_option(rust_type) {
                report.push(
                    Finding::error(
                        RuleId::DoubleOption,
                        spec_path,
                        format!("double-Option in rust_type: `{}`", rust_type),
                    )
                    .with_line(idx + 1),
                );
            }
        }
    }
}

/// Pull the value after `rust_type:` from a YAML line. Returns `None` if
/// the line isn't a rust_type assignment. Handles both quoted and unquoted
/// values; strips trailing comments and whitespace.
fn extract_rust_type_value(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("rust_type:")?;
    let rest = rest.trim();
    // Strip quotes if present. YAML allows "..." or '...' or bare.
    let rest = rest.trim_matches(|c| c == '"' || c == '\'');
    // Drop trailing comment (`# ...`). Rust types don't contain `#`.
    let rest = rest.split('#').next()?.trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest)
    }
}

/// Check whether a Rust-type string textually contains `Option<Option<`.
/// Uses a regex to tolerate whitespace (`Option <Option<` would still hit).
fn contains_double_option(rust_type: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"Option\s*<\s*Option\s*<").unwrap());
    re.is_match(rust_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        DoubleOptionRule {}.check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    #[test]
    fn clean_spec_emits_no_findings() {
        let spec = r#"---
id: foo
---

```yaml
schemas:
  - rust_type: HTTPException
    properties:
      status_code:
        rust_type: u16
```
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn plain_option_is_fine() {
        let spec = r#"```yaml
rust_type: Option<String>
```"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn double_option_is_flagged() {
        let spec = r#"```yaml
rust_type: Option<Option<u16>>
```"#;
        let report = run(spec);
        assert_eq!(report.findings.len(), 1);
        let f = &report.findings[0];
        assert_eq!(f.rule, RuleId::DoubleOption);
        assert_eq!(f.line, Some(2));
        assert!(f.message.contains("Option<Option<u16>>"));
    }

    #[test]
    fn double_option_with_whitespace_is_flagged() {
        let spec = r#"rust_type: Option < Option< String > >"#;
        let report = run(spec);
        assert_eq!(report.findings.len(), 1);
    }

    #[test]
    fn double_option_with_quotes_is_flagged() {
        let spec = r#"rust_type: "Option<Option<String>>""#;
        assert_eq!(run(spec).findings.len(), 1);
    }

    #[test]
    fn multiple_offending_lines_all_flagged() {
        let spec = r#"rust_type: Option<Option<u16>>
rust_type: String
rust_type: Option<Option<u32>>
"#;
        assert_eq!(run(spec).findings.len(), 2);
    }

    #[test]
    fn rust_type_inside_nested_generic_is_not_double_option() {
        // Triple-nested Vec<Option<T>> etc. is fine.
        let spec = r#"rust_type: Vec<Option<String>>
rust_type: HashMap<String, Option<u16>>
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn non_rust_type_lines_ignored() {
        let spec = r#"name: Option<Option<Foo>>  # not a rust_type: line
description: example Option<Option<T>> in prose
"#;
        assert!(run(spec).is_empty());
    }

    #[test]
    fn trailing_comment_stripped_before_match() {
        let spec = r#"rust_type: Option<String>  # not double"#;
        assert!(run(spec).is_empty());
    }
}
/// DoubleOptionRule validation rule (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/validate/rules/r3a_double_option.md#schema
pub struct DoubleOptionRule {}
````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r3a_double_option.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
