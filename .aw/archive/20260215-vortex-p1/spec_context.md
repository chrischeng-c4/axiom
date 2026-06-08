---
change_id: vortex-p1
type: spec_context
created_at: 2026-02-14T16:40:42.751279+00:00
updated_at: 2026-02-14T16:40:42.751279+00:00
iteration: 2
complexity: high
stage: spec
scanned_groups:
  - cclab-aurora
  - cclab-cli
  - cclab-core
  - cclab-genesis
  - cclab-grid
  - cclab-grid-db
  - cclab-ion
  - cclab-mamba
  - cclab-meteor
  - cclab-nebula
  - cclab-nova
  - cclab-nucleus
  - cclab-orbit
  - cclab-photon
  - cclab-prism
  - cclab-probe
  - cclab-pulsar-array-core
  - cclab-quasar
  - cclab-server
  - cclab-shield
  - cclab-titan
  - cclab-vortex
  - nebula
---

# Spec Context

## Relevant Specs

- **vortex-core-architecture** (group: cclab-vortex)
  - relevance: high
  - reason: Defines the core engine lifecycle which must be extended into a full game state machine.
  - key sections: R1: VortexEngine Lifecycle Interface Contract, Core Loop Thread and Lifecycle Sequence diagram
- **vortex-render-wgpu** (group: cclab-vortex)
  - relevance: high
  - reason: Foundation for rendering; must be extended for layered rendering and text.
  - key sections: R1: Batch Rendering, R3: Camera System
- **vortex-ecs-engine** (group: cclab-vortex)
  - relevance: high
  - reason: Core data model and execution foundation for all P1 systems.
  - key sections: R1: Sparse Set Component Storage, R3: Parallel System Execution
- **vortex-agent-bt** (group: cclab-vortex)
  - relevance: high
  - reason: Current AI foundation; needs to be paired with FSM support.
  - key sections: R1: Composite Nodes, R3: Nova Agent Sync
- **vortex-td-mechanics** (group: cclab-vortex)
  - relevance: medium
  - reason: Specifies how components are composed into gameplay; provides context for state machine and AI.
  - key sections: R2: Turret Logic, Wave Lifecycle State Machine diagram
- **architecture** (group: cclab-orbit)
  - relevance: medium
  - reason: Potential reuse of async/event patterns for Internal Event Bus.
  - key sections: Orbit Event Loop Architecture
- **state-machine** (group: cclab-probe)
  - relevance: medium
  - reason: Pattern for Level/Game state machine and state transitions.
  - key sections: Probe Test Runner State Machine
- **02-architecture-principles** (group: cclab-core)
  - relevance: medium
  - reason: GIL handling and Rust/Python boundary principles critical for engine workers.
  - key sections: GIL Release Strategy, Parallel Processing

## Dependencies

- vortex-game-state depends on vortex-event-bus
- vortex-render-layers depends on vortex-render-wgpu
- vortex-text-renderer depends on vortex-render-layers
- vortex-input-interaction depends on vortex-event-bus and vortex-render-layers
- All gameplay systems depend on vortex-ecs-engine

## Gaps

- No dedicated spec for Internal Event Bus (#345). Needs to support sync/async patterns.
- Existing vortex-core-architecture is limited to engine lifecycle; needs extended Game State Machine (#371) for Loading, LevelSelect, etc.
- vortex-agent-bt only covers Behavior Trees; needs FSM support to fulfill the 'Both' requirement for Tower AI (#334).
- vortex-render-wgpu lacks explicit details for Layered Rendering (Z-ordering) and Bitmap Text Rendering.
- No existing specs for Input Mapping and Player Interaction systems.
