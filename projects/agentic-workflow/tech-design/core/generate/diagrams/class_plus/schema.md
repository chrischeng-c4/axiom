---
id: sdd-generate-class-plus-schema
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Class Plus Schema

## Overview
<!-- type: overview lang: markdown -->

Mermaid-Plus class diagram definition types in
`projects/agentic-workflow/src/generate/diagrams/class_plus/schema.rs`. Ten serde shapes:

- `ClassDiagramDef` — top-level diagram (id, classes map, relationships, namespaces, optional description).
- `ClassDef` — one class (optional name, optional stereotype, attributes, methods, optional description).
- `ClassStereotype` — 7-variant enum with `serde_rename_all: lowercase`.
- `AttributeDef` — one attribute (name, JSON-key `type` mapped to `attr_type`, visibility, is_static, optional default_value, optional description).
- `MethodDef` — one method (name, parameters, optional return_type, visibility, is_static, is_abstract, optional description).
- `ParameterDef` — one parameter (name, JSON-key `type` mapped to `param_type`).
- `Visibility` — 4-variant enum with `serde_rename_all: lowercase`, default Public.
- `RelationshipDef` — one class↔class link (from, to, JSON-key `type` mapped to `rel_type`, optional label/multiplicities).
- `RelationshipType` — 6-variant enum with `serde_rename_all: lowercase`.
- `NamespaceDef` — one namespace grouping (name, classes Vec).

Codegen emits the serde import used by generated derives and attributes.
Hand-written outside CODEGEN: module docstring, non-serde `HashMap` import,
and the `#[cfg(test)] mod tests` block.

This spec exercises:

1. **`x-serde-rename`** — `attr_type`, `param_type`, `rel_type` map to JSON keys `"type"`.
2. **Default variant** — `Visibility::Public` uses `is_default: true`.
3. **`x-serde-default`** on optional fields.
4. **`HashMap<K, V>` field** — `classes: HashMap<String, ClassDef>` via x-rust-type.
5. **Vec of inner type** — `attributes: Vec<AttributeDef>` etc.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ClassStereotype:
    type: string
    enum: [Interface, Abstract, Enumeration, Service, Entity, ValueObject, Aggregate]
    description: Class stereotype.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
      serde_rename_all: lowercase

  Visibility:
    type: string
    enum: [Public, Private, Protected, Package]
    description: Visibility modifier.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: lowercase
      variants:
        - { name: Public, is_default: true, doc: "Public visibility (default)." }
        - { name: Private, doc: "Private visibility." }
        - { name: Protected, doc: "Protected visibility." }
        - { name: Package, doc: "Package-private visibility." }

  RelationshipType:
    type: string
    enum: [Inheritance, Composition, Aggregation, Association, Dependency, Realization]
    description: Relationship type between two classes.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
      serde_rename_all: lowercase

  ClassDiagramDef:
    type: object
    required: [id, classes, relationships, namespaces, description]
    description: Class diagram definition.
    properties:
      id:
        type: string
        description: "Diagram identifier."
      classes:
        type: object
        x-rust-type: "HashMap<String, ClassDef>"
        description: "Class definitions keyed by class name."
      relationships:
        type: array
        items: { $ref: "#/definitions/RelationshipDef" }
        x-rust-type: "Vec<RelationshipDef>"
        x-serde-default: true
        description: "Relationships between classes."
      namespaces:
        type: array
        items: { $ref: "#/definitions/NamespaceDef" }
        x-rust-type: "Vec<NamespaceDef>"
        x-serde-default: true
        description: "Namespace/package groupings."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Diagram description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ClassDef:
    type: object
    required: [name, stereotype, attributes, methods, description]
    description: Class definition.
    properties:
      name:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Display name (optional, defaults to key)."
      stereotype:
        type: string
        x-rust-type: "Option<ClassStereotype>"
        x-serde-default: true
        description: "Class stereotype."
      attributes:
        type: array
        items: { $ref: "#/definitions/AttributeDef" }
        x-rust-type: "Vec<AttributeDef>"
        x-serde-default: true
        description: "Attributes."
      methods:
        type: array
        items: { $ref: "#/definitions/MethodDef" }
        x-rust-type: "Vec<MethodDef>"
        x-serde-default: true
        description: "Methods."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  AttributeDef:
    type: object
    required: [name, attr_type, visibility, is_static, default_value, description]
    description: Attribute definition.
    properties:
      name:
        type: string
        description: "Attribute name."
      attr_type:
        type: string
        x-serde-rename: "type"
        description: "Attribute type."
      visibility:
        type: string
        x-rust-type: "Visibility"
        x-serde-default: true
        description: "Visibility."
      is_static:
        type: boolean
        x-serde-default: true
        description: "Is static."
      default_value:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Default value."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  MethodDef:
    type: object
    required: [name, parameters, return_type, visibility, is_static, is_abstract, description]
    description: Method definition.
    properties:
      name:
        type: string
        description: "Method name."
      parameters:
        type: array
        items: { $ref: "#/definitions/ParameterDef" }
        x-rust-type: "Vec<ParameterDef>"
        x-serde-default: true
        description: "Parameters."
      return_type:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Return type."
      visibility:
        type: string
        x-rust-type: "Visibility"
        x-serde-default: true
        description: "Visibility."
      is_static:
        type: boolean
        x-serde-default: true
        description: "Is static."
      is_abstract:
        type: boolean
        x-serde-default: true
        description: "Is abstract."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ParameterDef:
    type: object
    required: [name, param_type]
    description: Parameter definition.
    properties:
      name:
        type: string
        description: "Parameter name."
      param_type:
        type: string
        x-serde-rename: "type"
        description: "Parameter type."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RelationshipDef:
    type: object
    required: [from, to, rel_type, label, from_multiplicity, to_multiplicity]
    description: Relationship definition.
    properties:
      from:
        type: string
        description: "Source class."
      to:
        type: string
        description: "Target class."
      rel_type:
        type: string
        x-rust-type: "RelationshipType"
        x-serde-rename: "type"
        description: "Relationship type."
      label:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Relationship label."
      from_multiplicity:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Source multiplicity."
      to_multiplicity:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Target multiplicity."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  NamespaceDef:
    type: object
    required: [name, classes]
    description: Namespace definition.
    properties:
      name:
        type: string
        description: "Namespace name."
      classes:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Classes in this namespace."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/class_plus/schema.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ClassDiagramDef
      - ClassDef
      - ClassStereotype
      - AttributeDef
      - MethodDef
      - ParameterDef
      - Visibility
      - RelationshipDef
      - RelationshipType
      - NamespaceDef
    description: |
      Codegen replaces all ten type declarations and emits the serde import
      required by their derives and attributes.
  - path: projects/agentic-workflow/src/generate/diagrams/class_plus/schema.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring,
      `use std::collections::HashMap;`, and the `#[cfg(test)] mod tests`
      block (~55 LOC).
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Ten serde shapes; mix of structs/enums; HashMap + Vec + Options.
- [schema] All well-formed; x-serde-rename for `type`-keyed fields, is_default for Visibility::Public.
- [changes] All ten in `replaces`; tests + module-level items hand-written.
