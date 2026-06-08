---
id: sdd-block-plus-schema
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Block Plus Schema

## Overview
<!-- type: overview lang: markdown -->

5 types defining Mermaid block-beta diagrams. First dogfood use of
`x-serde-default: "<fn_name>"` (commit 53ec4245) for `default_columns`
/ `default_width` helpers. Codegen emits the serde import required by
the declarations.

- BlockDef: top-level. columns field uses default_columns fn.
- BlockNodeDef: recursive (children: Vec<BlockNodeDef>). width uses
  default_width fn. metadata is Option<serde_json::Value>.
- BlockShape: 8-variant lowercase enum, is_default on Default variant.
- BlockEdgeDef: edges between blocks.
- BlockEdgeStyle: 3-variant lowercase enum, is_default on Arrow.

Hand-written: `fn default_columns()`, `fn default_width()`, tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  BlockShape:
    type: string
    enum: [Default, Round, Stadium, Diamond, Cylinder, Hexagon, Circle, Subroutine]
    description: Block shape types.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: lowercase
      variants:
        - { name: Default,    is_default: true,   doc: "Default rectangular." }
        - { name: Round,                          doc: "Round." }
        - { name: Stadium,                        doc: "Stadium." }
        - { name: Diamond,                        doc: "Diamond." }
        - { name: Cylinder,                       doc: "Cylinder." }
        - { name: Hexagon,                        doc: "Hexagon." }
        - { name: Circle,                         doc: "Circle." }
        - { name: Subroutine,                     doc: "Subroutine." }

  BlockEdgeStyle:
    type: string
    enum: [Arrow, Thick, Dotted]
    description: Edge style.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: lowercase
      variants:
        - { name: Arrow,    is_default: true,    doc: "Solid arrow (default)." }
        - { name: Thick,                         doc: "Thick arrow." }
        - { name: Dotted,                        doc: "Dotted arrow." }

  BlockDef:
    type: object
    required: [id, columns, blocks, edges]
    description: Block diagram definition (Mermaid block-beta).
    properties:
      id:
        type: string
      title:
        type: string
      columns:
        type: integer
        x-rust-type: u32
        x-serde-default: "default_columns"
      blocks:
        type: array
        items:
          $ref: "#/definitions/BlockNodeDef"
      edges:
        type: array
        items:
          $ref: "#/definitions/BlockEdgeDef"
        x-serde-default: true
      description:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  BlockNodeDef:
    type: object
    required: [id, label, shape, width, children]
    description: Block node definition.
    properties:
      id:
        type: string
      label:
        type: string
      shape:
        $ref: "#/definitions/BlockShape"
        x-serde-default: true
      width:
        type: integer
        x-rust-type: u32
        x-serde-default: "default_width"
      children:
        type: array
        items:
          $ref: "#/definitions/BlockNodeDef"
        x-serde-default: true
      child_columns:
        type: integer
        x-rust-type: "Option<u32>"
        x-serde-default: true
      metadata:
        type: object
        x-rust-type: "Option<serde_json::Value>"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  BlockEdgeDef:
    type: object
    required: [from, to, style]
    description: Edge between blocks.
    properties:
      from:
        type: string
      to:
        type: string
      label:
        type: string
      style:
        $ref: "#/definitions/BlockEdgeStyle"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/block_plus/schema.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - BlockDef
      - BlockNodeDef
      - BlockShape
      - BlockEdgeDef
      - BlockEdgeStyle
    description: Codegen replaces all 5 type declarations.
  - path: projects/agentic-workflow/src/generate/diagrams/block_plus/schema.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module preamble, non-serde use statements,
      `fn default_columns()`, `fn default_width()`, tests block.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] First x-serde-default fn-name dogfood + recursive Vec<BlockNodeDef>.
- [schema] is_default + lowercase + custom default fn names all proven.
- [changes] codegen + hand-written split correct.
