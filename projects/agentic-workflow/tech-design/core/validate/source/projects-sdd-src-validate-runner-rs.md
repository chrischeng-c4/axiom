---
id: projects-sdd-src-validate-runner-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/runner.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validate/runner.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `run_rules` | projects/agentic-workflow/src/validate/runner.rs | function | pub | 18 | run_rules(spec_paths: &[PathBuf]) -> RuleReport |
## Source
<!-- type: source lang: rust -->

`````rust
//! Rule runner — read spec file(s), dispatch every registered rule, collect
//! all findings into one report.
//!
//! This is the read-only path used by `validate <prefix>` and
//! `validate <file>`. Slug mode (commit-gate) lives in the aw binary because
//! it has to write the git trailer and advance phase — out of scope here.

use crate::validate::rules::all_rules;
use crate::validate::RuleReport;
use std::path::{Path, PathBuf};

/// Run the full rule registry against every spec in `spec_paths`. Returns a
/// merged report. A per-file read error surfaces as a `ReadError` finding so
/// a bad path doesn't silently drop the file.
pub fn run_rules(spec_paths: &[PathBuf]) -> RuleReport {
    let rules = all_rules();
    let mut report = RuleReport::new();
    for path in spec_paths {
        run_rules_on_file(path, &rules, &mut report);
    }
    report
}

fn run_rules_on_file(
    path: &Path,
    rules: &[Box<dyn crate::validate::Rule>],
    report: &mut RuleReport,
) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            // Non-rule failure. Surface as a synthetic finding tagged R3a
            // (arbitrary pick; the runner never knows which rule "owns" a
            // read error). Callers will see the file + message and can act.
            report.push(crate::validate::Finding::error(
                crate::validate::RuleId::DoubleOption,
                path,
                format!("failed to read spec file: {}", e),
            ));
            return;
        }
    };
    let checkable_content = mask_source_section_bodies(&content);
    for rule in rules {
        rule.check(path, &checkable_content, report);
    }
}

fn mask_source_section_bodies(content: &str) -> String {
    let mut masked = Vec::new();
    let mut source_annotation_pending = false;
    let mut source_fence_close: Option<String> = None;

    for line in content.lines() {
        if let Some(close) = &source_fence_close {
            if fence_closes(line, close) {
                source_fence_close = None;
                masked.push(line.to_string());
            } else {
                masked.push(String::new());
            }
            continue;
        }

        masked.push(line.to_string());

        let trimmed = line.trim();
        if trimmed.starts_with("<!--")
            && trimmed.contains("type:")
            && trimmed.contains("source")
        {
            source_annotation_pending = true;
            continue;
        }

        if source_annotation_pending {
            if let Some(close) = fence_close_marker(line) {
                source_fence_close = Some(close);
                source_annotation_pending = false;
            } else if line.starts_with("## ") {
                source_annotation_pending = false;
            }
        }
    }

    let mut out = masked.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn fence_close_marker(line: &str) -> Option<String> {
    let first = line.as_bytes().first().copied()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let count = line.as_bytes().iter().take_while(|b| **b == first).count();
    if count < 3 {
        return None;
    }
    Some(line[..count].to_string())
}

fn fence_closes(line: &str, opener: &str) -> bool {
    let Some(marker) = fence_close_marker(line) else {
        return false;
    };
    marker.as_bytes().first() == opener.as_bytes().first()
        && marker.len() >= opener.len()
        && line[marker.len()..].trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn clean_spec_produces_no_findings() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("clean.md");
        std::fs::write(
            &file,
            r#"---
id: clean
---

## Overview
<!-- type: overview lang: markdown -->

Nothing to lint.
"#,
        )
        .unwrap();
        let report = run_rules(&[file]);
        assert!(
            report.is_empty(),
            "clean spec should produce no findings, got: {:#?}",
            report.findings,
        );
    }

    #[test]
    fn double_option_spec_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("bad.md");
        std::fs::write(
            &file,
            r#"---
id: bad
---

```yaml
rust_type: Option<Option<u16>>
```
"#,
        )
        .unwrap();
        let report = run_rules(&[file]);
        assert!(!report.is_empty());
        assert!(report.has_errors());
        assert!(report
            .findings
            .iter()
            .any(|f| matches!(f.rule, crate::validate::RuleId::DoubleOption)));
    }

    #[test]
    fn missing_file_surfaces_finding_not_panic() {
        let report = run_rules(&[PathBuf::from("/nonexistent/spec.md")]);
        assert!(!report.is_empty());
        assert!(report.findings[0].message.contains("failed to read"));
    }

    #[test]
    fn multiple_files_all_checked() {
        let tmp = tempfile::tempdir().unwrap();
        let clean = tmp.path().join("a.md");
        let bad = tmp.path().join("b.md");
        std::fs::write(&clean, "---\nid: a\n---\n").unwrap();
        std::fs::write(
            &bad,
            r#"---
id: b
---

```yaml
rust_type: Option<Option<String>>
```
"#,
        )
        .unwrap();
        let report = run_rules(&[clean, bad]);
        assert_eq!(report.findings.len(), 1);
    }

    #[test]
    fn source_section_fixtures_are_not_linted() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("source.md");
        std::fs::write(
            &file,
            r##"---
id: source
---

## Source
<!-- type: source lang: rust -->

````rust
let bad_fixture = r#"
```yaml
rust_type: Option<Option<u16>>
```
"#;
````
"##,
        )
        .unwrap();
        let report = run_rules(&[file]);
        assert!(
            report.is_empty(),
            "source fixtures should be masked, got: {:#?}",
            report.findings
        );
    }
}
`````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/runner.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs.
```
