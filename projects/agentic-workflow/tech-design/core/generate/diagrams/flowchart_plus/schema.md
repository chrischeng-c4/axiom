---
id: sdd-generate-flowchart-plus-schema
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Flowchart Plus Schema

## Overview
<!-- type: overview lang: markdown -->

Mermaid-Plus flowchart definition types in
`projects/agentic-workflow/src/generate/diagrams/flowchart_plus/schema.rs`. Eleven serde
shapes:

- `PrimitiveKind` — registry-backed primitive operation enum used by
  `NodeDef.primitive`; this folds the currently implemented subset of
  `mermaid-plus-primitive-vocabulary.md` into the replayable flowchart schema.
- `FlowchartDef` — top-level (id, direction, nodes IndexMap, edges, subgraphs, optional description).
- `FlowDirection` — 4-variant enum UPPERCASE, default TB.
- `NodeDef` — label, shape (default skip), optional semantic, optional
  description, optional primitive binding, primitive args map, optional output
  binding.
- `NodeShape` — 10-variant enum lowercase, default Rectangle.
- `SemanticType` — 11-variant **internally-tagged** enum (`tag = "type"`, snake_case), mix of unit + struct variants.
- `DbOperation` — 4-variant enum UPPERCASE.
- `HttpMethod` — 5-variant enum UPPERCASE.
- `EdgeDef` — from, to, optional label, style (default skip), optional condition, is_error_path bool.
- `EdgeStyle` — 3-variant enum lowercase, default Arrow.
- `SubgraphDef` — id, label, nodes Vec<String>, optional description.

Codegen emits the serde import used by generated derives and attributes.
Hand-written outside CODEGEN: module docstring, non-serde imports, the
`is_default_direction`, `is_default_shape`, `is_false`,
`is_default_edge_style` predicate functions, and the
`#[cfg(test)] mod tests` block.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  PrimitiveKind:
    type: string
    enum:
      - ReadFile
      - WriteFile
      - AppendFile
      - PathExists
      - ParseJsonlStream
      - AppendLineAtomic
      - ParseJsonlStr
      - SerializeJsonlLine
      - RunSubprocess
      - ParseYaml
      - ParseJson
      - SerializeYaml
      - FormatTemplate
      - Now
      - TtyCheck
      - PrintStdout
      - Call
    description: |
      Named primitive operations for flowchart YAML nodes.
      Each variant maps to a single Rust emit template in PrimitiveRegistry.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash]
      serde_rename_all: snake_case

  FlowDirection:
    type: string
    enum: [TB, BT, LR, RL]
    description: Flow direction.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: UPPERCASE
      variants:
        - { name: TB, is_default: true, doc: "Top to bottom (default)." }
        - { name: BT, doc: "Bottom to top." }
        - { name: LR, doc: "Left to right." }
        - { name: RL, doc: "Right to left." }

  NodeShape:
    type: string
    enum: [Rectangle, Rounded, Stadium, Subroutine, Cylinder, Circle, Diamond, Hexagon, Parallelogram, Trapezoid]
    description: Node shape.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: lowercase
      variants:
        - { name: Rectangle, is_default: true, doc: "Rectangle (default)." }
        - { name: Rounded, doc: "Rounded rectangle." }
        - { name: Stadium, doc: "Stadium shape." }
        - { name: Subroutine, doc: "Subroutine shape." }
        - { name: Cylinder, doc: "Cylinder (database) shape." }
        - { name: Circle, doc: "Circle." }
        - { name: Diamond, doc: "Diamond (decision)." }
        - { name: Hexagon, doc: "Hexagon." }
        - { name: Parallelogram, doc: "Parallelogram (input/output)." }
        - { name: Trapezoid, doc: "Trapezoid." }

  DbOperation:
    type: string
    enum: [Insert, Update, Delete, Upsert]
    description: Database operation type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
      serde_rename_all: UPPERCASE

  HttpMethod:
    type: string
    enum: [Get, Post, Put, Patch, Delete]
    description: HTTP method.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
      serde_rename_all: UPPERCASE

  EdgeStyle:
    type: string
    enum: [Arrow, Thick, Dotted]
    description: Edge style.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: lowercase
      variants:
        - { name: Arrow, is_default: true, doc: "Arrow (default)." }
        - { name: Thick, doc: "Thick line." }
        - { name: Dotted, doc: "Dotted line." }

  SemanticType:
    type: string
    enum: [Start, End, Validation, Condition, DbQuery, DbMutation, ApiCall, Transform, Assign, RaiseError, LoopStart, LoopEnd]
    description: Semantic type for code generation.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
      serde_tag: "type"
      serde_rename_all: snake_case
      variants:
        - { name: Start, doc: "Start node." }
        - name: End
          kind: struct
          doc: "End/return node."
          fields:
            - { name: output, rust_type: "Option<String>" }
        - name: Validation
          kind: struct
          doc: "Input validation."
          fields:
            - { name: input, rust_type: String }
            - { name: rules, rust_type: "Vec<String>" }
            - { name: error_code, rust_type: "Option<i32>" }
            - { name: error_message, rust_type: "Option<String>" }
        - name: Condition
          kind: struct
          doc: "Condition/decision."
          fields:
            - { name: expression, rust_type: String }
        - name: DbQuery
          kind: struct
          doc: "Database query (SELECT)."
          fields:
            - { name: table, rust_type: String }
            - { name: filter, rust_type: "Option<String>" }
            - { name: output, rust_type: "Option<String>" }
        - name: DbMutation
          kind: struct
          doc: "Database mutation (INSERT/UPDATE/DELETE)."
          fields:
            - { name: operation, rust_type: DbOperation }
            - { name: table, rust_type: String }
            - { name: data, rust_type: "Option<String>" }
        - name: ApiCall
          kind: struct
          doc: "External API call."
          fields:
            - { name: method, rust_type: HttpMethod }
            - { name: url, rust_type: String }
            - { name: body, rust_type: "Option<String>" }
            - { name: output, rust_type: "Option<String>" }
        - name: Transform
          kind: struct
          doc: "Data transformation."
          fields:
            - { name: input, rust_type: String }
            - { name: output, rust_type: String }
            - { name: expression, rust_type: "Option<String>" }
        - name: Assign
          kind: struct
          doc: "Variable assignment."
          fields:
            - { name: variable, rust_type: String }
            - { name: value, rust_type: String }
        - name: RaiseError
          kind: struct
          doc: "Raise error."
          fields:
            - { name: code, rust_type: i32 }
            - { name: message, rust_type: String }
        - name: LoopStart
          kind: struct
          doc: "Loop start."
          fields:
            - { name: condition, rust_type: String }
        - { name: LoopEnd, doc: "Loop end." }

  FlowchartDef:
    type: object
    required: [id, direction, nodes, edges, subgraphs, description]
    description: Flowchart definition (input from LLM).
    properties:
      id:
        type: string
        description: "Diagram identifier."
      direction:
        type: string
        x-rust-type: "FlowDirection"
        x-serde-default: true
        x-serde-skip-if: "is_default_direction"
        description: "Flow direction."
      nodes:
        type: object
        x-rust-type: "IndexMap<String, NodeDef>"
        description: "Node definitions keyed by node ID."
      edges:
        type: array
        items: { type: object }
        x-rust-type: "Vec<EdgeDef>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Edge definitions."
      subgraphs:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SubgraphDef>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Subgraph definitions."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Diagram description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  NodeDef:
    type: object
    required: [label, shape, semantic, description, primitive, args, output]
    description: Node definition.
    properties:
      label:
        type: string
        description: "Node display label."
      shape:
        type: string
        x-rust-type: "NodeShape"
        x-serde-default: true
        x-serde-skip-if: "is_default_shape"
        description: "Node shape."
      semantic:
        type: object
        x-rust-type: "Option<SemanticType>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Semantic type for code generation."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Node description."
      primitive:
        type: string
        x-rust-type: "Option<PrimitiveKind>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: |
          Primitive operation: when present the logic generator uses the named
          primitive's emit template rather than generic scaffolding.
      args:
        type: object
        x-rust-type: "HashMap<String, serde_yaml::Value>"
        x-serde-default: true
        x-serde-skip-if: "HashMap::is_empty"
        description: |
          Primitive input bindings: maps input field names to upstream
          variable names or literal values.
      output:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: |
          Rust variable name to which this node's output is bound.
          Downstream nodes reference it by this name.
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, Default]

  EdgeDef:
    type: object
    required: [from, to, label, style, condition, is_error_path]
    description: Edge definition.
    properties:
      from:
        type: string
        description: "Source node ID."
      to:
        type: string
        description: "Target node ID."
      label:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Edge label."
      style:
        type: string
        x-rust-type: "EdgeStyle"
        x-serde-default: true
        x-serde-skip-if: "is_default_edge_style"
        description: "Edge style."
      condition:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Semantic: condition expression (for conditional branches)."
      is_error_path:
        type: boolean
        x-serde-default: true
        x-serde-skip-if: "is_false"
        description: "Semantic: is this an error path?"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SubgraphDef:
    type: object
    required: [id, label, nodes, description]
    description: Subgraph definition.
    properties:
      id:
        type: string
        description: "Subgraph identifier."
      label:
        type: string
        description: "Subgraph display label."
      nodes:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Node IDs contained in this subgraph."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Optional description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/flowchart_plus/schema.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - PrimitiveKind
      - FlowchartDef
      - FlowDirection
      - NodeDef
      - NodeShape
      - SemanticType
      - DbOperation
      - HttpMethod
      - EdgeDef
      - EdgeStyle
      - SubgraphDef
    description: |
      Codegen replaces all eleven type declarations and emits the serde import
      required by their derives and attributes.
  - path: projects/agentic-workflow/src/generate/diagrams/flowchart_plus/schema.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring, non-serde imports
      (`indexmap::IndexMap`, `std::collections::HashMap`), the
      `is_default_direction`, `is_default_shape`, `is_false`,
      `is_default_edge_style` predicate functions, and the
      `#[cfg(test)] mod tests` block.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Ten serde shapes; mix of structs/enums; SemanticType is the complex piece (internally-tagged with mix of unit + struct variants).
- [schema] All well-formed; serde_tag + serde_rename_all combined for SemanticType; default variants via is_default; predicates referenced via x-serde-skip-if.
- [changes] All ten in `replaces`; predicates + tests + module-level items hand-written.
