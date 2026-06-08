// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#tests
// CODEGEN-BEGIN

//! Integration test (R7): parse a multi-section TD spec, dispatch through
//! `TDAst`, and assert the dispatcher classifies each section.
//!
//! @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#logic
use agentic_workflow::generate::from_td_ast::{dispatch_from_tdast, DispatchCtx, DispatchStatus};
use agentic_workflow::td_ast::parse::parse_td;

fn fixture_spec() -> String {
    r#"---
id: stage2-dispatch-fixture
fill_sections: [schema, changes]
---

# Fixture for Stage 2 dispatch

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: stage2-dispatch-fixture#schema
title: Fixture
definitions:
  Foo:
    type: object
    properties:
      name: { type: string }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
$id: stage2-dispatch-fixture#changes
changes:
  - path: projects/agentic-workflow/src/foo.rs
    action: create
    description: "Foo emitter."
```
"#
    .to_string()
}

#[test]
fn dispatch_classifies_each_section() {
    let tmpdir = tempfile::tempdir().expect("tempdir");
    let spec_path = tmpdir.path().join("fixture.md");
    std::fs::write(&spec_path, fixture_spec()).expect("write fixture");

    let td_ast = parse_td(&spec_path).expect("parse_td");
    assert_eq!(td_ast.sections.len(), 2, "fixture has 2 sections");

    let ctx = DispatchCtx {
        spec_path: spec_path.clone(),
        spec_ref_prefix: spec_path.display().to_string(),
        target_lang: None,
    };

    let report = dispatch_from_tdast(&td_ast, &ctx);
    assert_eq!(report.outcomes.len(), 2, "one DispatchOutcome per section");

    // Schema -> rust-schema generator (currently LegacyFallback in Stage 2).
    let schema_outcome = report
        .outcomes
        .iter()
        .find(|o| o.section_type == "schema")
        .expect("schema outcome");
    assert_eq!(schema_outcome.generator, "rust-schema");
    assert_eq!(schema_outcome.status, DispatchStatus::LegacyFallback);
}

#[test]
fn empty_spec_yields_empty_report() {
    let tmpdir = tempfile::tempdir().expect("tempdir");
    let spec_path = tmpdir.path().join("empty.md");
    std::fs::write(
        &spec_path,
        "---\nid: empty\nfill_sections: []\n---\n\n# Empty\n",
    )
    .expect("write empty");

    let td_ast = parse_td(&spec_path).expect("parse_td empty");
    let ctx = DispatchCtx {
        spec_path: spec_path.clone(),
        spec_ref_prefix: spec_path.display().to_string(),
        target_lang: None,
    };
    let report = dispatch_from_tdast(&td_ast, &ctx);
    assert!(report.outcomes.is_empty());
    assert!(report.orphan_changes_paths.is_empty());
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END
