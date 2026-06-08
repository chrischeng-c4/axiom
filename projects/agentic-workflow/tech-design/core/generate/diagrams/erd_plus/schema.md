---
id: sdd-generate-erd-plus-schema
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# ERD Plus Schema

## Overview
<!-- type: overview lang: markdown -->

Mermaid-Plus ERD definition types in
`projects/agentic-workflow/src/generate/diagrams/erd_plus/schema.rs`. Six serde shapes:

- `ERDDef` — top-level diagram (id, entities map, relationships, optional description).
- `EntityDef` — one entity table (optional name, attributes, optional description).
- `ERDAttributeDef` — one column (name, JSON-key `type` mapped to Rust `data_type`, optional key/references/comment, nullable bool).
- `KeyType` — 3-variant enum (PK, FK, UK), default identifier serde shape.
- `ERDRelationshipDef` — one entity↔entity link (from, to, cardinality, optional label, identifying bool).
- `Cardinality` — 8-variant enum with `serde_rename_all: kebab-case`.

Codegen emits the serde import used by generated derives and attributes.
Hand-written outside CODEGEN: module docstring, the
`is_false(v: &bool) -> bool { !v }` predicate function (used by several
`skip_serializing_if = "is_false"` attrs), and the `#[cfg(test)] mod tests`
block.

This spec is the first dogfood use of `x-serde-rename` (commit
`0e9c3d2a`) on `ERDAttributeDef.data_type` to map Rust field
`data_type` → JSON key `"type"`.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  KeyType:
    type: string
    enum: [PK, FK, UK]
    description: Attribute key type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
      variants:
        - { name: PK, doc: "Primary key." }
        - { name: FK, doc: "Foreign key." }
        - { name: UK, doc: "Unique key." }

  Cardinality:
    type: string
    enum:
      - OneToOne
      - OneToMany
      - ManyToOne
      - ManyToMany
      - OneOrMoreToOne
      - OneToOneOrMore
      - ZeroOrOneToOne
      - OneToZeroOrOne
    description: Relationship cardinality between two entities.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
      serde_rename_all: kebab-case

  ERDDef:
    type: object
    required: [id, entities, relationships]
    description: ERD definition.
    properties:
      id:
        type: string
        description: "Diagram identifier."
      entities:
        type: object
        x-rust-type: "indexmap::IndexMap<String, EntityDef>"
        description: "Entity definitions keyed by entity name."
      relationships:
        type: array
        items:
          $ref: "#/definitions/ERDRelationshipDef"
        description: "Relationships between entities."
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
      description:
        type: string
        description: "Diagram description."
        x-serde-skip-if: "Option::is_none"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  EntityDef:
    type: object
    required: [attributes]
    description: Entity definition.
    properties:
      name:
        type: string
        description: "Display name (optional)."
        x-serde-skip-if: "Option::is_none"
      attributes:
        type: array
        items:
          $ref: "#/definitions/ERDAttributeDef"
        description: "Attributes."
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
      description:
        type: string
        description: "Description."
        x-serde-skip-if: "Option::is_none"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ERDAttributeDef:
    type: object
    required: [name, data_type, nullable]
    description: Attribute definition.
    properties:
      name:
        type: string
        description: "Attribute name."
      data_type:
        type: string
        x-serde-rename: "type"
        description: "Data type (mapped to JSON key 'type' — Rust reserved word)."
      key:
        $ref: "#/definitions/KeyType"
        description: "Key type."
        x-serde-skip-if: "Option::is_none"
      nullable:
        type: boolean
        description: "Is nullable."
        x-serde-default: true
        x-serde-skip-if: "is_false"
      references:
        type: string
        description: "Foreign key reference (entity.attribute)."
        x-serde-skip-if: "Option::is_none"
      comment:
        type: string
        description: "Comment/description."
        x-serde-skip-if: "Option::is_none"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ERDRelationshipDef:
    type: object
    required: [from, to, cardinality, identifying]
    description: Relationship definition.
    properties:
      from:
        type: string
        description: "Source entity."
      to:
        type: string
        description: "Target entity."
      cardinality:
        $ref: "#/definitions/Cardinality"
      label:
        type: string
        description: "Relationship label."
        x-serde-skip-if: "Option::is_none"
      identifying:
        type: boolean
        description: "Is identifying relationship."
        x-serde-default: true
        x-serde-skip-if: "is_false"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/erd_plus/schema.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - KeyType
      - Cardinality
      - ERDDef
      - EntityDef
      - ERDAttributeDef
      - ERDRelationshipDef
    description: |
      Codegen replaces all 6 type declarations and emits the serde import
      required by their derives and attributes.
  - path: projects/agentic-workflow/src/generate/diagrams/erd_plus/schema.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring,
      `fn is_false(v: &bool) -> bool { !v }` predicate (referenced by
      `skip_serializing_if = "is_false"` on `nullable` and `identifying`),
      and the `#[cfg(test)] mod tests` block.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] All six types are fully specified with correct serde extensions: `x-serde-rename: "type"` on `ERDAttributeDef.data_type` (R3), `x-rust-type` on `ERDDef.entities` (R2), `x-serde-skip-if: "is_false"` on boolean fields (R4/R8), `serde_rename_all: kebab-case` on `Cardinality` (R6), and bare-variant serialization on `KeyType` (R5).
- [changes] Two-entry split (codegen block covering all 6 symbols + hand-written boundary for `is_false` predicate and test block) correctly satisfies R7 and R8.
