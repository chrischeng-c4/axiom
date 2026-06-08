---
id: sdd-generate-mindmap-plus-schema
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Mindmap Plus Schema

## Overview
<!-- type: overview lang: markdown -->

Mermaid-Plus mindmap definition types in
`projects/agentic-workflow/src/generate/diagrams/mindmap_plus/schema.rs`. Three serde
shapes:

- `MindmapDef` — top-level diagram (id, root node, optional description).
- `MindmapNodeDef` — recursive tree node (label, shape, optional icon,
  child nodes, optional description).
- `MindmapShapePlus` — 6-variant unit enum naming the node shape with
  container-level `serde_rename_all: lowercase`. `Square` carries
  `#[default]` so an absent JSON shape key deserialises as `Square`.

This is the first dogfood use of `is_default: true` on an enum variant
(commit 0e9c3d2a). Codegen emits the serde import used by generated derives
and attributes. No impl blocks; nothing hand-written outside CODEGEN beyond
the file's module docstring and test module.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  MindmapShapePlus:
    type: string
    enum: [Square, Rounded, Circle, Bang, Cloud, Hexagon]
    description: |
      Mermaid-Plus mindmap node shape. Default is Square so an absent
      JSON shape key deserialises to a sensible visual.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: lowercase
      variants:
        - { name: Square, is_default: true, doc: "Square node (default)." }
        - { name: Rounded, doc: "Rounded square node." }
        - { name: Circle, doc: "Circular node." }
        - { name: Bang, doc: "Bang (callout) node." }
        - { name: Cloud, doc: "Cloud node." }
        - { name: Hexagon, doc: "Hexagonal node." }

  MindmapDef:
    type: object
    required: [id, root]
    description: Mindmap definition.
    properties:
      id:
        type: string
        description: "Diagram identifier."
      root:
        $ref: "#/definitions/MindmapNodeDef"
        description: "Root node."
      description:
        type: string
        description: "Diagram description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  MindmapNodeDef:
    type: object
    required: [label, shape, children]
    description: Mindmap node definition (recursive).
    properties:
      label:
        type: string
        description: "Node label."
      shape:
        $ref: "#/definitions/MindmapShapePlus"
        description: "Node shape; absent key defaults to Square."
        x-serde-default: true
      icon:
        type: string
        description: "Icon (emoji or text)."
      children:
        type: array
        items:
          $ref: "#/definitions/MindmapNodeDef"
        description: "Child nodes."
        x-serde-default: true
      description:
        type: string
        description: "Description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/mindmap_plus/schema.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - MindmapShapePlus
      - MindmapDef
      - MindmapNodeDef
    description: |
      Codegen replaces all 3 type declarations and emits the serde import
      required by their derives and attributes.
  - path: projects/agentic-workflow/src/generate/diagrams/mindmap_plus/schema.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring and
      `#[cfg(test)] mod tests` block (preserved verbatim).
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Accurately describes all three types, notes the recursive `children` field, and calls out the `is_default` dogfood commit. No gaps.
- [schema] All fields across `MindmapDef`, `MindmapNodeDef`, and `MindmapShapePlus` are present with correct required/optional partitioning, `$ref` self-reference, `x-serde-default` annotations, and derive lists matching requirements R2–R4.
- [changes] Two-entry split (codegen + hand-written) with `replaces` list is complete and aligns with R5. No ambiguity for the codegen pipeline.
