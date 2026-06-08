---
id: sdd-fillback-graph
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Dependency Graph Types

## Overview
<!-- type: overview lang: markdown -->

Dependency graph types in `projects/agentic-workflow/src/fillback/graph.rs`. Five shapes:

- `DependencyType` — 3-variant enum, lowercase rename. Derives `[Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash]`.
- `Dependency` — `from`, `to`, `dependency_type`. Derives `[Debug, Clone, Serialize, Deserialize]`.
- `ModuleNode` — `name`, `path`, `is_external: bool`, `symbol_count: usize`, `public_symbol_count: usize`. Derives `[Debug, Clone, Serialize, Deserialize]`.
- `DependencyGraph` — `nodes: Vec<ModuleNode>`, `edges: Vec<Dependency>`. Derives `[Debug, Clone, Serialize, Deserialize]`.
- `GraphStats` — `total_modules`, `internal_modules`, `external_dependencies`, `total_edges` (usize), `avg_dependencies_per_module: f64`, `most_connected_modules: Vec<(String, usize)>`. Derives `[Debug, Clone, Serialize, Deserialize]`.

Codegen replaces all five type declarations and the serde import they need.
Companion source specs own the non-serde imports, display implementation,
graph construction/rendering helpers, stats helpers, and tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  DependencyType:
    type: string
    enum: [Import, Call, Inheritance]
    description: Dependency type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash]
      serde_rename_all: lowercase

  Dependency:
    type: object
    required: [from, to, dependency_type]
    description: A single dependency edge.
    properties:
      from:
        type: string
        description: "Source module name."
      to:
        type: string
        description: "Target module name."
      dependency_type:
        type: string
        x-rust-type: "DependencyType"
        description: "Kind of dependency."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ModuleNode:
    type: object
    required: [name, path, is_external, symbol_count, public_symbol_count]
    description: Module node in the dependency graph.
    properties:
      name:
        type: string
        description: "Module name."
      path:
        type: string
        description: "Module path."
      is_external:
        type: boolean
        description: "Whether the module is external to the crate."
      symbol_count:
        type: integer
        x-rust-type: "usize"
        description: "Total symbol count in the module."
      public_symbol_count:
        type: integer
        x-rust-type: "usize"
        description: "Public symbol count in the module."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  DependencyGraph:
    type: object
    required: [nodes, edges]
    description: Complete dependency graph.
    properties:
      nodes:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ModuleNode>"
        description: "Graph nodes."
      edges:
        type: array
        items: { type: object }
        x-rust-type: "Vec<Dependency>"
        description: "Graph edges."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  GraphStats:
    type: object
    required: [total_modules, internal_modules, external_dependencies, total_edges, avg_dependencies_per_module, most_connected_modules]
    description: Aggregate stats for a dependency graph.
    properties:
      total_modules:
        type: integer
        x-rust-type: "usize"
        description: "Total module count."
      internal_modules:
        type: integer
        x-rust-type: "usize"
        description: "Internal module count."
      external_dependencies:
        type: integer
        x-rust-type: "usize"
        description: "External dependency count."
      total_edges:
        type: integer
        x-rust-type: "usize"
        description: "Total edge count."
      avg_dependencies_per_module:
        type: number
        x-rust-type: "f64"
        description: "Average dependencies per module."
      most_connected_modules:
        type: array
        items: { type: object }
        x-rust-type: "Vec<(String, usize)>"
        description: "Top connected modules with edge counts."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/graph.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - DependencyType
      - Dependency
      - ModuleNode
      - DependencyGraph
      - GraphStats
    description: |
      Codegen replaces all five type declarations and the generated serde import.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] 5 standard data shapes; one enum + 4 structs.
- [schema] All in `required:`; foreign-type fields via x-rust-type incl tuple type Vec<(String, usize)>.
- [changes] Standard split.
