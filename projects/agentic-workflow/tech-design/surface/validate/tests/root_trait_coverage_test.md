---
id: projects-agentic-workflow-tests-cli-tests-root-trait-coverage-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: partial
    rationale: "Bidirectional coverage between doc_mirror::TRAITS and CONTRIBUTING.md's Service archetype section (issue #1077, archetype-as-traits slice 1/3): proves every trait anchor resolves to a real heading, and every archetype H3 is either trait-anchored or explicitly marked policy-only."
---

# Standardized projects/agentic-workflow/tests/cli/tests/root_trait_coverage_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/root_trait_coverage_test.rs`.

### Symbols

No public AST symbols.

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/root_trait_coverage_test.rs -->
```rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/root_trait_coverage_test.md#source
// CODEGEN-BEGIN
//! Bidirectional coverage between `doc_mirror::TRAITS` and CONTRIBUTING.md's
//! "Service archetype" section (issue #1077, archetype-as-traits slice 1/3).
//!
//! Two directions, both checked against the SAME `doc_mirror::TRAITS`
//! constant that `aw init`'s trait-table projector consumes
//! (`crate::cli::doc_mirror::render_trait_table`), so the generator and this
//! checker can never disagree (issue #1077 AC3):
//!
//! (a) every [`TraitDef`]-equivalent `contributing_anchor` must resolve to a
//!     real heading line in CONTRIBUTING.md — an anchor that stops existing
//!     (heading renamed/removed) fails loudly instead of silently linking
//!     nowhere.
//! (b) every H3 inside the "## Service archetype..." section must either be
//!     one of those trait anchors, or carry a literal `policy-only` marker
//!     line in its own body — so a new archetype subsection can never
//!     silently fall outside both "trait-enforced" and "explicitly
//!     judgment-only".

use agentic_workflow::cli::doc_mirror::TRAITS;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .expect("repo root")
        .to_path_buf()
}

/// Line-index range `[start, end)` of the "## Service archetype..." section:
/// from its own heading line up to (exclusive) the next top-level `## `
/// heading, or end of file when it is the last section.
fn service_archetype_section_range(lines: &[&str]) -> (usize, usize) {
    let start = lines
        .iter()
        .position(|l| l.starts_with("## Service archetype"))
        .expect("CONTRIBUTING.md must contain a `## Service archetype` section");
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.starts_with("## "))
        .map(|off| start + 1 + off)
        .unwrap_or(lines.len());
    (start, end)
}

/// AC3(a): every trait's `contributing_anchor` must resolve to a real
/// heading line in CONTRIBUTING.md.
#[test]
fn every_trait_contributing_anchor_resolves_to_a_real_heading() {
    let root = repo_root();
    let contributing =
        fs::read_to_string(root.join("CONTRIBUTING.md")).expect("read CONTRIBUTING.md");
    let lines: Vec<&str> = contributing.lines().collect();

    for def in TRAITS {
        assert!(
            lines.iter().any(|l| *l == def.contributing_anchor),
            "trait `{}`'s contributing_anchor `{}` does not match any heading in CONTRIBUTING.md \
             (heading renamed/removed?)",
            def.id,
            def.contributing_anchor
        );
    }
}

/// AC3(b): every H3 within the "## Service archetype..." section either
/// matches a trait's anchor, or its own body (before the next H3 or the end
/// of the section) contains a literal `policy-only` marker line.
#[test]
fn every_service_archetype_h3_is_either_trait_anchored_or_marked_policy_only() {
    let root = repo_root();
    let contributing =
        fs::read_to_string(root.join("CONTRIBUTING.md")).expect("read CONTRIBUTING.md");
    let lines: Vec<&str> = contributing.lines().collect();
    let (section_start, section_end) = service_archetype_section_range(&lines);

    let h3_indices: Vec<usize> = (section_start + 1..section_end)
        .filter(|&i| lines[i].starts_with("### "))
        .collect();
    assert!(
        !h3_indices.is_empty(),
        "the `## Service archetype` section must contain at least one `### ` heading"
    );

    let trait_anchors: Vec<&str> = TRAITS.iter().map(|def| def.contributing_anchor).collect();

    for (pos, &h3_at) in h3_indices.iter().enumerate() {
        let heading = lines[h3_at];
        if trait_anchors.contains(&heading) {
            continue;
        }

        let body_end = h3_indices.get(pos + 1).copied().unwrap_or(section_end);
        let body = lines[h3_at + 1..body_end].join("\n");
        assert!(
            body.contains("policy-only"),
            "heading `{heading}` is neither a trait anchor ({trait_anchors:?}) nor marked \
             `policy-only` in its body — add a trait, or an explicit \
             `*(policy-only — ...)*` marker line"
        );
    }
}
// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/root_trait_coverage_test.rs
    action: create
    section: source
    description: |
      Issue #1077 (traits slice 1/3): bidirectional coverage test between
      `agentic_workflow::cli::doc_mirror::TRAITS` and CONTRIBUTING.md's
      "## Service archetype..." section, both directions imported from/
      checked against the SAME production constant `aw init`'s trait-table
      projector consumes, so the generator and this checker can never
      disagree (AC3). `every_trait_contributing_anchor_resolves_to_a_real_heading`
      proves direction (a): every `TraitDef.contributing_anchor` must match
      a real heading line in CONTRIBUTING.md, so a renamed/removed heading
      fails the suite instead of silently producing a dead link.
      `every_service_archetype_h3_is_either_trait_anchored_or_marked_policy_only`
      proves direction (b): every `### ` heading within the archetype
      section's line range must either equal a trait's anchor, or its own
      body (up to the next H3 or section end) must contain a literal
      `policy-only` marker line — added to the three pure-judgment sections
      ("The shared service kit", "Deploy tenancy", "Service dogfood rules")
      and the three trait-pending sections ("OpenAPI client codegen",
      "Standard endpoints", "EC gates", each noting `trait pending — #1078`)
      so the test passes honestly today and #1078 can flip the
      trait-pending markers to real anchors later.
    impl_mode: hand-written
```
