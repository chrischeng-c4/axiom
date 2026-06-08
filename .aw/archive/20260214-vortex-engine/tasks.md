---
id: vortex-engine
change_id: vortex-engine
type: tasks
version: 1
created_at: 2026-02-14T06:56:17.467214+00:00
updated_at: 2026-02-14T06:57:22+00:00
proposal_ref: vortex-engine
summary:
  total: 13
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 13
layers:
  data:
    task_count: 1
    estimated_files: 2
  logic:
    task_count: 4
    estimated_files: 4
  integration:
    task_count: 2
    estimated_files: 3
  testing:
    task_count: 6
    estimated_files: 1
history:
  - timestamp: 2026-02-14T06:56:17.467214+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
  - timestamp: 2026-02-14T06:57:22+00:00
    agent: "codex"
    tool: "manual-edit"
    action: "revised-paths-and-layering"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 13 implementation tasks for change `vortex-engine`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Data Layer | 1 | Pending |
| Logic Layer | 4 | Pending |
| Integration Layer | 2 | Pending |
| Testing Layer | 6 | Pending |

## 1. Data Layer

### Task 1.1: Update Vortex crate manifest and workspace membership

```yaml
id: 1.1
action: UPDATE
status: pending
file: crates/cclab-vortex/Cargo.toml
additional_files: [Cargo.toml]
spec_ref: vortex-core-architecture:*
```

Create the crate data layer entry for `cclab-vortex` and add workspace membership in root `Cargo.toml`.

## 2. Logic Layer

### Task 2.1: Implement Vortex Core Architecture module

```yaml
id: 2.1
action: UPDATE
status: pending
file: crates/cclab-vortex/src/core/mod.rs
spec_ref: vortex-core-architecture:*
depends_on: [1.1]
```

Implement Vortex Core Architecture & Lifecycle covering:
- R1: VortexEngine Lifecycle Interface Contract
- R2: Threading Model and GIL Boundary
- R3: Frame-Rate Independence with Hybrid Time Step

### Task 2.2: Implement Vortex ECS engine module

```yaml
id: 2.2
action: UPDATE
status: pending
file: crates/cclab-vortex/src/ecs/mod.rs
spec_ref: vortex-ecs-engine:*
depends_on: [2.1]
```

Implement Vortex ECS Engine & Component Storage covering:
- R1: Sparse Set Component Storage (High-performance iteration)
- R2: Query System (Filtering entities by component composition)
- R3: Parallel System Execution (Safe concurrent access to disjoint components)

### Task 2.3: Implement Vortex Behavior Tree AI module

```yaml
id: 2.3
action: UPDATE
status: pending
file: crates/cclab-vortex/src/agent/mod.rs
spec_ref: vortex-agent-bt:*
depends_on: [2.2]
```

Implement Vortex Behavior Tree AI System covering:
- R1: Composite Nodes
- R2: Blackboard System
- R3: Nova Agent Sync

### Task 2.4: Implement Vortex Tower Defense mechanics module

```yaml
id: 2.4
action: UPDATE
status: pending
file: crates/cclab-vortex/src/td/mod.rs
spec_ref: vortex-td-mechanics:*
depends_on: [2.2, 3.1, 2.3]
```

Implement Vortex Tower Defense Gameplay Mechanics covering:
- R1: Wave Spawner
- R2: Turret Logic
- R3: Grid-based Pathfinding

## 3. Integration Layer

### Task 3.1: Implement Vortex WGPU render module

```yaml
id: 3.1
action: UPDATE
status: pending
file: crates/cclab-vortex/src/render/mod.rs
spec_ref: vortex-render-wgpu:*
depends_on: [2.1]
```

Implement Vortex 2D WGPU Rendering Pipeline covering:
- R1: Batch Rendering
- R2: Tilemap System
- R3: Camera System

### Task 3.2: Implement Vortex MCP module and server router integration

```yaml
id: 3.2
action: UPDATE
status: pending
file: crates/cclab-vortex/src/mcp/mod.rs
additional_files: [crates/cclab-server/src/mcp/router.rs]
spec_ref: vortex-mcp-integration:*
depends_on: [2.1]
```

Implement Vortex MCP Integration & Dynamic Tool Registry covering:
- R1: Dynamic Registry
- R2: Tool Prefixes
- R3: Backwards Compatibility

## 4. Testing Layer

### Task 4.1: Add tests for Vortex Core Architecture & Lifecycle

```yaml
id: 4.1
action: CREATE
status: pending
file: crates/cclab-vortex/tests/integration.rs
spec_ref: vortex-core-architecture:*
depends_on: [2.1]
```

Create unit tests for Vortex Core Architecture & Lifecycle covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Vortex ECS Engine & Component Storage

```yaml
id: 4.2
action: CREATE
status: pending
file: crates/cclab-vortex/tests/integration.rs
spec_ref: vortex-ecs-engine:*
depends_on: [2.2]
```

Create unit tests for Vortex ECS Engine & Component Storage covering all requirements and acceptance scenarios

### Task 4.3: Add tests for Vortex 2D WGPU Rendering Pipeline

```yaml
id: 4.3
action: CREATE
status: pending
file: crates/cclab-vortex/tests/integration.rs
spec_ref: vortex-render-wgpu:*
depends_on: [3.1]
```

Create unit tests for Vortex 2D WGPU Rendering Pipeline covering all requirements and acceptance scenarios

### Task 4.4: Add tests for Vortex Behavior Tree AI System

```yaml
id: 4.4
action: CREATE
status: pending
file: crates/cclab-vortex/tests/integration.rs
spec_ref: vortex-agent-bt:*
depends_on: [2.3]
```

Create unit tests for Vortex Behavior Tree AI System covering all requirements and acceptance scenarios

### Task 4.5: Add tests for Vortex MCP Integration & Dynamic Tool Registry

```yaml
id: 4.5
action: CREATE
status: pending
file: crates/cclab-vortex/tests/integration.rs
spec_ref: vortex-mcp-integration:*
depends_on: [3.2]
```

Create integration tests for Vortex MCP routing behavior and dynamic registry compatibility gates

### Task 4.6: Add tests for Vortex Tower Defense Gameplay Mechanics

```yaml
id: 4.6
action: CREATE
status: pending
file: crates/cclab-vortex/tests/integration.rs
spec_ref: vortex-td-mechanics:*
depends_on: [2.4]
```

Create unit tests for Vortex Tower Defense Gameplay Mechanics covering all requirements and acceptance scenarios

</tasks>
