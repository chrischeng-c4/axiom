---
change_id: vortex-p1-batch
type: spec_context
created_at: 2026-02-14T10:06:53.034939+00:00
updated_at: 2026-02-14T10:06:53.034939+00:00
iteration: 2
complexity: high
stage: spec
scanned_groups:
  - vortex-engine
---

# Spec Context

## Relevant Specs

- **vortex-core-architecture** (group: vortex-engine)
  - relevance: high
  - reason: Defines the core loop where render and state machine integration occurs.
  - key sections: R1: Lifecycle Interface, R3: Hybrid Time Step Loop
- **vortex-ecs-engine** (group: vortex-engine)
  - relevance: high
  - reason: Core data layer for all gameplay and render-facing state.
  - key sections: R1: Sparse Set Storage, R3: Parallel Systems
- **vortex-render-wgpu** (group: vortex-engine)
  - relevance: high
  - reason: Directly impacted by render integration, layers, and camera control tasks.
  - key sections: R1: Batch Rendering, R3: Camera System
- **vortex-td-mechanics** (group: vortex-engine)
  - relevance: high
  - reason: Basis for TD tests and player interaction logic.
  - key sections: R1: Wave Spawner, R2: Turret Logic, R4: Resource Management
- **vortex-agent-bt** (group: vortex-engine)
  - relevance: high
  - reason: Foundational for agent-driven AI and Nova integration.
  - key sections: R3: Nova Agent Sync

## Dependencies

- vortex-core-architecture
- vortex-ecs-engine
- vortex-render-wgpu
- vortex-td-mechanics
- vortex-agent-bt

## Gaps

- Formal specification for internal Event Bus for ECS and non-ECS messaging.
- Global Game State Machine for high-level flow (MainMenu, Play, Pause, etc.).
- Detailed input mapping and interaction logic for player actions (build/sell).
- Bitmap font rendering specification for Debug/HUD overlays.
- Layered rendering architecture and Z-order management.
