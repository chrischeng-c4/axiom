---
id: libs-surface-tests-snapshot-rs
summary: Lossless rust-source-unit coverage for `libs/surface/tests/snapshot.rs`.
capability_refs:
  - id: renderer-neutral-ui-surface-model
    role: primary
    claim: renderer-neutral-ui-surface-model-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Surface library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/surface/tests/snapshot.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/surface/tests/snapshot.rs` captured during libs codegen standardization.

No public Rust symbols detected by the source-unit capture pass.


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
use cclab_surface::{Element, Props, SurfaceRect};

#[test]
fn snapshot_serializes_stable_structural_tree() {
    let element = Element::intrinsic(
        "label",
        Props {
            id: Some("project-name-label".to_string()),
            html_for: Some("project-name".to_string()),
            ..Default::default()
        },
        vec![
            Element::text("Project"),
            Element::intrinsic(
                "input",
                Props {
                    id: Some("project-name".to_string()),
                    value: Some("aw".to_string()),
                    ..Default::default()
                },
                vec![],
            ),
        ],
    );

    let mut snapshot = element.surface_snapshot();
    assert!(snapshot.set_bounds(
        "root/1",
        SurfaceRect {
            x: 8.0,
            y: 12.0,
            w: 120.0,
            h: 24.0,
        }
    ));

    let json = serde_json::to_value(&snapshot).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["nodes"][0]["semantic_id"], "project-name-label");
    assert_eq!(json["nodes"][0]["role"], "label");
    assert_eq!(json["nodes"][0]["name"], "Project");
    assert_eq!(json["nodes"][2]["semantic_id"], "project-name");
    assert_eq!(json["nodes"][2]["role"], "textbox");
    assert_eq!(json["nodes"][2]["name"], "aw");
    assert_eq!(json["nodes"][2]["bounds"]["w"], 120.0);
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/surface/tests/snapshot.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/surface/tests/snapshot.rs` captured during libs codegen standardization.
```
