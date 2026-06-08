---
id: vortex-ecs-engine
type: spec
title: "Vortex ECS Engine & Component Storage"
version: 1
spec_type: data-model
tags: [data]
created_at: 2026-02-14T06:45:09.191977+00:00
updated_at: 2026-02-14T06:45:09.191977+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: true
  has_pseudo_code: false
  has_api_spec: true
  has_semantic_diagrams: false
  api_spec_type: json-schema
  diagrams:
    - type: erd
      title: "ECS Entity and Component Storage Model"
    - type: class
      title: "World, Query, and System Interfaces"
depends:
  - vortex-core-architecture
changes:
  - file: crates/cclab-vortex/src/ecs/mod.rs
    action: create
    description: "Define ECS public interfaces for World, Query, System, and scheduler contracts."
  - file: crates/cclab-vortex/src/ecs/entity.rs
    action: create
    description: "Implement entity ID and generation-based lifecycle model."
  - file: crates/cclab-vortex/src/ecs/storage/sparse_set.rs
    action: create
    description: "Implement sparse-set component storage with dense iteration semantics."
  - file: crates/cclab-vortex/src/ecs/query.rs
    action: create
    description: "Implement query specification and filtering execution path."
  - file: crates/cclab-vortex/src/ecs/scheduler.rs
    action: create
    description: "Implement conflict-aware parallel execution planner for systems."
history:
  - timestamp: 2026-02-14T06:45:09.191977+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Vortex ECS Engine & Component Storage

## Overview

Define the ECS subsystem for Vortex, including entity lifecycle, sparse-set component storage, query composition, and parallel-safe system scheduling. This spec extends `vortex-core-architecture` by specifying the data layout and interfaces used by the core update loop for high-throughput simulation workloads.

## Requirements

### R1 - Sparse Set Component Storage (High-performance iteration)

```yaml
id: R1
priority: high
status: draft
```

The ECS must store components using sparse-set backed storage per component type to support O(1) average insert/remove/lookup by entity ID and cache-friendly dense iteration over active component rows. Storage must maintain entity/component index coherence during swaps and removals.

### R2 - Query System (Filtering entities by component composition)

```yaml
id: R2
priority: high
status: draft
```

The ECS must expose a query interface that filters entities by component composition (`with`, `without`, optional predicates) and returns iterators over matching dense rows. Query execution must avoid full-world scans and use component cardinality heuristics to choose an efficient driving set.

### R3 - Parallel System Execution (Safe concurrent access to disjoint components)

```yaml
id: R3
priority: high
status: draft
```

The ECS scheduler must support parallel execution of systems when their read/write component access sets are disjoint or read-compatible. The scheduler must detect write/write and read/write conflicts, enforce deterministic stage boundaries, and prevent unsafe concurrent mutable access to the same component storage.

## Acceptance Criteria

### Scenario: Sparse Set Dense Iteration

- **GIVEN** A world with 100,000 entities where 60,000 have `Position` and `Velocity` components stored in sparse sets.
- **WHEN** The movement system runs a query for entities with `Position` and `Velocity`.
- **THEN** The system iterates contiguous dense arrays without scanning unrelated entities, and each returned entity maps to valid component rows in O(1) average lookup.

### Scenario: Query Composition Filtering

- **GIVEN** A world containing entities with mixed `Transform`, `Renderable`, and `Hidden` components.
- **WHEN** A render query requests `with(Transform, Renderable)` and `without(Hidden)`.
- **THEN** Only entities satisfying the required composition are returned, and entities missing required components or containing excluded components are not included.

### Scenario: Conflict-Aware Parallel Scheduling

- **GIVEN** One system writes `Position`, a second reads `Position`, and a third writes `Health`.
- **WHEN** The scheduler builds an execution plan for the frame.
- **THEN** The `Position` writer and `Position` reader are not executed concurrently, while the `Health` writer can execute in parallel with non-conflicting systems, preserving memory safety and deterministic stage ordering.

## Diagrams

### ECS Entity and Component Storage Model

```mermaid
erDiagram
    EntityRecord {
        u32 PK id
        u32 generation
        bool alive
    }
    ComponentStorage {
        TypeId PK component_type
        enum<SparseSet> storage_kind
    }
    SparseSetRow {
        TypeId PK component_type
        u32 PK entity_id
        u32 dense_index
        bytes component_blob
    }
    QuerySpec {
        u64 PK id
        TypeId[] with_set
        TypeId[] without_set
    }
    SystemSpec {
        string PK name
        TypeId[] reads
        TypeId[] writes
        string stage
    }
    EntityRecord ||--o{ SparseSetRow : owns component rows
    ComponentStorage ||--o{ SparseSetRow : contains rows
    QuerySpec }o--o{ ComponentStorage : references type sets
    SystemSpec }o--o{ ComponentStorage : declares accesses
```

### World, Query, and System Interfaces

```mermaid
classDiagram
    class World {
        <<interface>>
        +spawn(Bundle components) Entity
        +despawn(Entity entity) bool
        +insert_component(Entity entity, T component) void
        +remove_component(Entity entity) Option<T>
        +query(QuerySpec spec) QueryResult
    }
    class ComponentStorage<T> {
        +insert(Entity entity, T component) void
        +remove(Entity entity) Option<T>
        +get(Entity entity) Option<&T>
        +get_mut(Entity entity) Option<&mut T>
        +iter_dense() Iterator<(Entity,&T)>
    }
    class SparseSet<T> {
        +Vec<u32> sparse
        +Vec<Entity> dense_entities
        +Vec<T> dense_components
    }
    class QuerySpec {
        +Vec<TypeId> with
        +Vec<TypeId> without
        +Vec<TypeId> maybe
    }
    class System {
        <<interface>>
        +name() &str
        +access() AccessSet
        +run(&mut World world) void
    }
    class Scheduler {
        +build_plan(Vec<System> systems) ExecutionPlan
        +execute(ExecutionPlan plan, &mut World world) void
    }
    World *-- ComponentStorage<T> : manages
    ComponentStorage<T> <|-- SparseSet<T> : implementation
    World --> QuerySpec : evaluates
    Scheduler --> System : orchestrates
```

## Data Model

```json
{
  "entity": {
    "properties": {
      "generation": {
        "format": "uint32",
        "type": "integer"
      },
      "id": {
        "format": "uint32",
        "type": "integer"
      }
    },
    "required": [
      "id",
      "generation"
    ],
    "type": "object"
  },
  "query_spec": {
    "properties": {
      "maybe": {
        "items": {
          "type": "string"
        },
        "type": "array"
      },
      "with": {
        "items": {
          "type": "string"
        },
        "type": "array"
      },
      "without": {
        "items": {
          "type": "string"
        },
        "type": "array"
      }
    },
    "required": [
      "with"
    ],
    "type": "object"
  },
  "sparse_set_storage": {
    "properties": {
      "dense_components": {
        "items": {
          "type": "object"
        },
        "type": "array"
      },
      "dense_entities": {
        "items": {
          "format": "uint32",
          "type": "integer"
        },
        "type": "array"
      },
      "sparse": {
        "items": {
          "format": "uint32",
          "type": "integer"
        },
        "type": "array"
      }
    },
    "required": [
      "sparse",
      "dense_entities",
      "dense_components"
    ],
    "type": "object"
  }
}
```

## API Specification (JSON Schema)

```yaml
$defs:
  Entity:
    additionalProperties: false
    properties:
      generation:
        minimum: 0
        type: integer
      id:
        minimum: 0
        type: integer
    required:
    - id
    - generation
    type: object
  QuerySpec:
    additionalProperties: false
    properties:
      maybe:
        items:
          type: string
        type: array
      with:
        items:
          type: string
        type: array
      without:
        items:
          type: string
        type: array
    required:
    - with
    type: object
  SparseSetStorage:
    additionalProperties: false
    properties:
      componentType:
        type: string
      denseComponents:
        items:
          type: object
        type: array
      denseEntities:
        items:
          minimum: 0
          type: integer
        type: array
      sparse:
        items:
          minimum: 0
          type: integer
        type: array
    required:
    - componentType
    - sparse
    - denseEntities
    - denseComponents
    type: object
  SystemAccess:
    additionalProperties: false
    properties:
      reads:
        items:
          type: string
        type: array
      stage:
        type: string
      writes:
        items:
          type: string
        type: array
    required:
    - reads
    - writes
    type: object
$id: https://cclab.dev/vortex/ecs.schema.json
$schema: https://json-schema.org/draft/2020-12/schema
properties:
  componentStorage:
    $ref: '#/$defs/SparseSetStorage'
  entity:
    $ref: '#/$defs/Entity'
  querySpec:
    $ref: '#/$defs/QuerySpec'
  systemAccess:
    $ref: '#/$defs/SystemAccess'
required:
- entity
- querySpec
title: Vortex ECS Data Model
type: object
```

</spec>
