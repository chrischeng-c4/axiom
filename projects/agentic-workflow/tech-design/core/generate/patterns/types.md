---
id: sdd-patterns-mod-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# UX Pattern Types

## Overview
<!-- type: overview lang: markdown -->

4 UX-pattern types in patterns/mod.rs. PatternNode is recursive. props
fields use HashMap<String, serde_json::Value>. Codegen emits the serde import
required by these declarations; the HashMap import stays hand-written.

Hand-written: PatternSource trait, pub mod registry/resolver, pub use re-exports.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  UxPattern:
    type: object
    required: [id, name, description, slots, layout]
    description: A reusable layout recipe that wireframe specs reference by ID.
    properties:
      id:
        type: string
      name:
        type: string
      description:
        type: string
      slots:
        type: array
        items:
          $ref: "#/definitions/PatternSlot"
      layout:
        type: array
        items:
          $ref: "#/definitions/PatternNode"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  PatternSlot:
    type: object
    required: [name, required, description]
    description: A named insertion point in the pattern's layout tree.
    properties:
      name:
        type: string
      required:
        type: boolean
        x-serde-default: true
      description:
        type: string
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  PatternNode:
    type: object
    required: [kind, props, children]
    description: A node in the abstract layout tree.
    properties:
      kind:
        type: string
      label:
        type: string
      slot_ref:
        type: string
      props:
        type: object
        x-rust-type: "HashMap<String, serde_json::Value>"
        x-serde-default: true
      children:
        type: array
        items:
          $ref: "#/definitions/PatternNode"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SlotContent:
    type: object
    required: [component, props]
    description: Content provided by a wireframe to fill a pattern slot.
    properties:
      component:
        type: string
      props:
        type: object
        x-rust-type: "HashMap<String, serde_json::Value>"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/patterns/mod.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - UxPattern
      - PatternSlot
      - PatternNode
      - SlotContent
    description: Codegen replaces 4 type declarations.
  - path: projects/agentic-workflow/src/generate/patterns/mod.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module preamble, pub mod registry/resolver,
      std::collections::HashMap import, PatternSource trait, pub use re-exports.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
