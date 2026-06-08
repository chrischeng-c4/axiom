---
id: vortex-agent-bt
type: spec
title: "Vortex Behavior Tree AI System"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-14T06:48:12.580017+00:00
updated_at: 2026-02-14T06:48:12.580017+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: true
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Behavior Tree Tick and Status Flow"
    - type: class
      title: "BehaviorTreeSystem and Node Interfaces"
depends:
  - vortex-ecs-engine
changes:
  - file: crates/cclab-vortex/src/agent/mod.rs
    action: create
    description: "Define public BT subsystem interfaces and system registration entrypoints."
  - file: crates/cclab-vortex/src/agent/bt/node.rs
    action: create
    description: "Define node kinds (Action, Condition, Selector, Sequence), node status, and execution contracts."
  - file: crates/cclab-vortex/src/agent/bt/blackboard.rs
    action: create
    description: "Implement scoped blackboard state model and typed key/value access APIs."
  - file: crates/cclab-vortex/src/agent/bt/system.rs
    action: create
    description: "Implement `BehaviorTreeSystem` ECS integration, per-frame tick ordering, and Nova sync boundary hooks."
history:
  - timestamp: 2026-02-14T06:48:12.580017+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Vortex Behavior Tree AI System

## Overview

Define the Behavior Tree (BT) AI subsystem for Vortex agents, including node semantics (Action, Condition, Selector, Sequence), blackboard state sharing, ECS integration, and explicit coordination boundaries with Nova high-level task agents. This spec extends `vortex-ecs-engine` by defining deterministic per-frame BT evaluation for ECS entities acting as agents.

## Requirements

### R1 - Composite Nodes

```yaml
id: R1
priority: high
status: draft
```

Behavior Trees must support composite nodes `Selector` and `Sequence` with deterministic left-to-right child evaluation and standard status propagation (`Success`, `Failure`, `Running`) to enable logic branching and staged decision behavior.

### R2 - Blackboard System

```yaml
id: R2
priority: high
status: draft
```

The BT runtime must provide a blackboard for shared state across BT nodes, with scoped keys (global, per-agent) and typed access semantics. Node execution must read/write blackboard values without violating ECS borrow-safety or causing frame-order nondeterminism.

### R3 - Nova Agent Sync

```yaml
id: R3
priority: high
status: draft
```

The BT subsystem must synchronize with Nova high-level task agents by consuming intent/goal updates from Nova and exposing BT execution status back to Nova. Sync must be explicit and bounded so Nova owns strategic planning while Vortex BT owns tactical per-frame behavior execution.

### R4 - Boundary Management

```yaml
id: R4
priority: high
status: draft
```

System boundaries must enforce that Vortex owns ECS-local simulation data, ticking, and low-latency behavior evaluation, while Nova owns cross-agent orchestration, long-horizon planning, and external tool workflows. The integration contract must prevent duplicated authority over movement/targeting decisions within the same frame.

## Acceptance Criteria

### Scenario: Selector and Sequence Status Propagation

- **GIVEN** An agent tree has a `Selector` with child A returning `Failure` and child B returning `Success`, and a `Sequence` with child C returning `Success` then child D returning `Running`.
- **WHEN** `BehaviorTreeSystem` ticks both trees for the frame.
- **THEN** The selector returns `Success` from child B after evaluating left-to-right, and the sequence returns `Running` at child D while preserving cursor state for the next frame tick.

### Scenario: Blackboard Shared State Access

- **GIVEN** A condition node checks `target_visible` and an action node updates `last_known_target_position` in the same agent blackboard scope.
- **WHEN** The tree is executed during an ECS update stage.
- **THEN** Both nodes access consistent typed blackboard values within the frame, writes are visible to subsequent nodes in deterministic order, and no illegal concurrent mutable access occurs.

### Scenario: ECS Agent Integration

- **GIVEN** Entities with `BehaviorTreeComponent`, `BlackboardComponent`, and gameplay components are registered as agents.
- **WHEN** `BehaviorTreeSystem` runs in the simulation loop.
- **THEN** Only entities matching required BT components are ticked, node actions can read/write ECS components through declared access sets, and non-agent entities are excluded.

### Scenario: Nova-Vortex Boundary Enforcement

- **GIVEN** Nova sends a high-level intent `defend_lane_A` and Vortex receives it before the frame tick.
- **WHEN** The BT executes tactical decisions for movement and target selection.
- **THEN** Vortex applies tactical behavior locally and reports status (`Running`/`Success`/`Failure`) back to Nova without Nova directly mutating ECS tactical state during that same tick.

## Diagrams

### Behavior Tree Tick and Status Flow

```mermaid
flowchart TB
    frame_start[Frame Tick Start]
    query_agents[Query ECS Agent Entities]
    load_context[Load Agent BT + Blackboard Context]
    eval_node{Evaluate Current Node} 
    node_action[Action / Condition Execution]
    composite_eval[Selector / Sequence Child Evaluation]
    status_success([Status: Success])
    status_failure([Status: Failure])
    status_running([Status: Running])
    sync_nova[Publish BT Status to Nova Sync Channel]
    frame_end[Frame Tick End]
    frame_start --> query_agents
    query_agents --> load_context
    load_context --> eval_node
    eval_node -->|leaf| node_action
    eval_node -->|composite| composite_eval
    node_action -->|Success| status_success
    node_action -->|Failure| status_failure
    node_action -->|Running| status_running
    composite_eval -->|Success| status_success
    composite_eval -->|Failure| status_failure
    composite_eval -->|Running| status_running
    status_success --> sync_nova
    status_failure --> sync_nova
    status_running --> sync_nova
    sync_nova --> frame_end
```

### BehaviorTreeSystem and Node Interfaces

```mermaid
classDiagram
    class BehaviorTreeSystem {
        <<interface>>
        +tick(World world, f32 delta_time) void
        +sync_with_nova(Entity agent_id, NodeStatus status) void
    }
    class BehaviorNode {
        <<interface>>
        +evaluate(BtContext ctx) NodeStatus
    }
    class SelectorNode {
        +evaluate(BtContext ctx) NodeStatus
    }
    class SequenceNode {
        +evaluate(BtContext ctx) NodeStatus
    }
    class ActionNode {
        +evaluate(BtContext ctx) NodeStatus
    }
    class ConditionNode {
        +evaluate(BtContext ctx) NodeStatus
    }
    class Blackboard {
        +get(BlackboardKey key) Option<Value>
        +set(BlackboardKey key, Value value) void
    }
    class BtContext {
        -Entity entity
        -Blackboard blackboard
        -World world
    }
    BehaviorTreeSystem *-- BehaviorNode : ticks
    SelectorNode <|-- BehaviorNode
    SequenceNode <|-- BehaviorNode
    ActionNode <|-- BehaviorNode
    ConditionNode <|-- BehaviorNode
    BehaviorNode --> BtContext : evaluate(ctx)
    BtContext --> Blackboard : shared state
```

## Data Model

```json
{
  "agent_bt_state": {
    "properties": {
      "entity": {
        "type": "integer"
      },
      "last_status": {
        "$ref": "#/node_status"
      },
      "tree_root": {
        "type": "string"
      }
    },
    "required": [
      "entity",
      "tree_root",
      "last_status"
    ],
    "type": "object"
  },
  "blackboard_entry": {
    "properties": {
      "key": {
        "type": "string"
      },
      "scope": {
        "enum": [
          "global",
          "agent"
        ],
        "type": "string"
      },
      "value": {
        "type": "object"
      }
    },
    "required": [
      "scope",
      "key",
      "value"
    ],
    "type": "object"
  },
  "node_kind": {
    "enum": [
      "Action",
      "Condition",
      "Selector",
      "Sequence"
    ],
    "type": "string"
  },
  "node_status": {
    "enum": [
      "Success",
      "Failure",
      "Running"
    ],
    "type": "string"
  }
}
```

</spec>
