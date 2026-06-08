---
id: projects-sdd-src-generate-tests-handwrite-scaffold-test-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/tests/handwrite_scaffold_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/tests/handwrite_scaffold_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/tests/handwrite_scaffold_test.rs -->
```rust
//! Unit tests for the HANDWRITE scaffold inserter.
//!
//! Covers R1, R3, R6, plus the gap / tracker / reason derivation
//! branches of the `scaffold-handwrite` flowchart.
//!
//! @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#test-plan

use std::path::PathBuf;
use tempfile::TempDir;

use crate::generate::handwrite::HandwriteEntry;
use crate::generate::handwrite_scaffold::{scaffold_handwrite, ScaffoldOutcome};

fn write(dir: &std::path::Path, rel: &str, body: &str) -> PathBuf {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn scaffold_emit_inserts_xml_marker_pair_around_anchor() {
    let tmp = TempDir::new().unwrap();
    let src = "pub fn hello() {\n    let x = 1;\n}\n";
    let p = write(tmp.path(), "src/x.rs", src);

    let entry = HandwriteEntry {
        gap: None,
        tracker: None,
        reason: Some("test reason".to_string()),
    };
    let out = scaffold_handwrite(&entry, &p, "hello", Some("logic")).unwrap();
    assert_eq!(out, ScaffoldOutcome::Inserted);

    let after = std::fs::read_to_string(&p).unwrap();
    assert!(
        after.contains("<HANDWRITE"),
        "open marker missing: {}",
        after
    );
    assert!(after.contains("</HANDWRITE>"), "close marker missing");
    assert!(after.contains("gap=\"missing-generator:logic\""));
    assert!(after.contains("tracker=\"pending-tracker\""));
    assert!(after.contains("reason=\"test reason\""));
}

#[test]
fn scaffold_idempotent_skips_when_marker_already_present() {
    let tmp = TempDir::new().unwrap();
    let src = "pub fn hello() {\n}\n";
    let p = write(tmp.path(), "src/x.rs", src);

    let entry = HandwriteEntry::default();
    let first = scaffold_handwrite(&entry, &p, "hello", Some("logic")).unwrap();
    assert_eq!(first, ScaffoldOutcome::Inserted);

    let second = scaffold_handwrite(&entry, &p, "hello", Some("logic")).unwrap();
    assert_eq!(
        second,
        ScaffoldOutcome::Skipped,
        "second call must be no-op"
    );

    let count = std::fs::read_to_string(&p)
        .unwrap()
        .matches("<HANDWRITE")
        .count();
    assert_eq!(count, 1, "duplicate markers detected");
}

#[test]
fn scaffold_default_tracker_is_pending_tracker() {
    let tmp = TempDir::new().unwrap();
    let p = write(tmp.path(), "src/x.rs", "pub struct Foo;\n");
    let entry = HandwriteEntry::default();
    scaffold_handwrite(&entry, &p, "Foo", Some("schema")).unwrap();
    let after = std::fs::read_to_string(&p).unwrap();
    assert!(after.contains("tracker=\"pending-tracker\""));
}

#[test]
fn scaffold_explicit_tracker_overrides_default() {
    let tmp = TempDir::new().unwrap();
    let p = write(tmp.path(), "src/x.rs", "pub struct Foo;\n");
    let entry = HandwriteEntry {
        gap: None,
        tracker: Some("issue-foo-bar".to_string()),
        reason: Some("explicit".to_string()),
    };
    scaffold_handwrite(&entry, &p, "Foo", Some("schema")).unwrap();
    let after = std::fs::read_to_string(&p).unwrap();
    assert!(after.contains("tracker=\"issue-foo-bar\""));
    assert!(!after.contains("pending-tracker"));
}

#[test]
fn scaffold_explicit_gap_overrides_section_derivation() {
    let tmp = TempDir::new().unwrap();
    let p = write(tmp.path(), "src/x.rs", "pub struct Foo;\n");
    let entry = HandwriteEntry {
        gap: Some("missing-primitive:flux-capacitor".to_string()),
        tracker: None,
        reason: Some("r".to_string()),
    };
    scaffold_handwrite(&entry, &p, "Foo", Some("schema")).unwrap();
    let after = std::fs::read_to_string(&p).unwrap();
    assert!(after.contains("gap=\"missing-primitive:flux-capacitor\""));
}

#[test]
fn scaffold_reason_synthesis_when_absent() {
    let tmp = TempDir::new().unwrap();
    let p = write(tmp.path(), "src/widget.rs", "pub fn run() {}\n");
    let entry = HandwriteEntry::default();
    scaffold_handwrite(&entry, &p, "run", Some("logic")).unwrap();
    let after = std::fs::read_to_string(&p).unwrap();
    // synthesised reason references the section + filename
    assert!(after.contains("reason="));
    assert!(
        after.contains("widget.rs"),
        "synthesised reason should include file: {}",
        after
    );
}

#[test]
fn scaffold_anchor_missing_returns_outcome_without_writing() {
    let tmp = TempDir::new().unwrap();
    let body = "pub fn other() {}\n";
    let p = write(tmp.path(), "src/x.rs", body);
    let entry = HandwriteEntry::default();
    let out = scaffold_handwrite(&entry, &p, "missing_symbol", Some("logic")).unwrap();
    assert_eq!(out, ScaffoldOutcome::AnchorMissing);
    assert_eq!(std::fs::read_to_string(&p).unwrap(), body);
}

#[test]
fn scaffold_round_trips_through_parser() {
    let tmp = TempDir::new().unwrap();
    let p = write(tmp.path(), "src/x.rs", "pub fn alpha() {\n}\n");
    let entry = HandwriteEntry {
        gap: Some("missing-generator:logic".to_string()),
        tracker: None,
        reason: Some("round-trip test".to_string()),
    };
    scaffold_handwrite(&entry, &p, "alpha", Some("logic")).unwrap();
    let body = std::fs::read_to_string(&p).unwrap();
    let parsed = crate::generate::parse_handwrite_markers(&body, "x.rs").unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].reason, "round-trip test");
    assert_eq!(parsed[0].tracker, "pending-tracker");
    assert_eq!(parsed[0].gap, "missing-generator:logic");
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/tests/handwrite_scaffold_test.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete handwrite scaffold test module.
```
