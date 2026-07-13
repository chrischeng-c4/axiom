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

# Standardized apps/agentic-workflow/src/validate/runner.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/validate/runner.rs` generated from AST during Score force-regeneration standardization.

The runner validates encoded source-partition controls against the original,
unmasked TD before ordinary rules receive a Source-masked view. Corrupt bounds,
ordering, base64 payloads, or digests therefore surface as the canonical
`SectionFormat` finding instead of disappearing with embedded fixtures.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `run_rules` | apps/agentic-workflow/src/validate/runner.rs | function | pub | 18 | run_rules(spec_paths: &[PathBuf]) -> RuleReport |
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
    // Source bodies are masked before the ordinary registry runs so embedded
    // fixtures do not trigger unrelated rules. Partition controls are the
    // exception: validate their complete, unmasked Source section first,
    // because masking the sentinel fence would otherwise hide corruption (or
    // make every valid partitioned artifact look corrupt). Tag the finding as
    // SectionFormat so `aw td check` exposes one canonical structural gate.
    if let Err(error) = crate::generate::apply::decode_partitioned_source(&content) {
        report.push(crate::validate::Finding::error(
            crate::validate::RuleId::SectionFormat,
            path,
            format!("invalid source partition manifest: {error}"),
        ));
    }
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
        if trimmed.starts_with("<!--") && trimmed.contains("type:") && trimmed.contains("source") {
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

    #[test]
    fn registry_rejects_corrupt_partition_manifest_before_source_masking() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let source_path = root.join("src/direct.py");
        let output_dir = root.join("tech-design/specs");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();
        std::fs::write(&source_path, "def direct():\n    return 42\n").unwrap();
        let outcome = crate::fillback::code::CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();
        let spec_path = outcome.spec_path.unwrap();

        let valid = run_rules(std::slice::from_ref(&spec_path));
        assert!(
            !valid.findings.iter().any(|finding| {
                finding.rule == crate::validate::RuleId::SectionFormat
                    && finding
                        .message
                        .contains("invalid source partition manifest")
            }),
            "valid partition controls must survive the runner's Source masking path: {:#?}",
            valid.findings
        );

        let content = std::fs::read_to_string(&spec_path).unwrap();
        let corrupted = content.replacen("digest=sha256:", "digest=sha256:0", 1);
        std::fs::write(&spec_path, corrupted).unwrap();
        let report = run_rules(&[spec_path]);
        assert!(report.findings.iter().any(|finding| {
            finding.rule == crate::validate::RuleId::SectionFormat
                && finding
                    .message
                    .contains("invalid source partition manifest")
        }));
    }
}
`````
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/validate/runner.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the remaining validation module source directly from the
      source section. Existing schema CODEGEN blocks, when present, remain
      owned by their semantic specs. Issue #1506 checks lossless source
      partition controls before Source masking and adds valid/corrupt manifest
      regression coverage.
```
