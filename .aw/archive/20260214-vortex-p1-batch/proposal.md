---
id: vortex-p1-batch
type: proposal
version: 2
created_at: 2026-02-14T10:28:12.958875+00:00
updated_at: 2026-02-14T10:28:12.958875+00:00
iteration: 1
scope: minor
spec_plan:
  - id: vortex-event-bus
    title: "Internal Event Bus System"
    depends: []
    context_refs:
      codebase: ["crates/cclab-vortex/src/core/app.rs"]
      spec: ["vortex-core-architecture"]
      knowledge: ["orbit/performance-tuning.md"]
    affected_code: ["crates/cclab-vortex/src/core/event.rs"]
  - id: vortex-render-layers
    title: "Layered Rendering and Camera Integration"
    depends: [vortex-event-bus]
    context_refs:
      codebase: ["crates/cclab-vortex/src/render/sprite.rs"]
      spec: ["vortex-render-wgpu"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 3 }
    affected_code: ["crates/cclab-vortex/src/render/layers.rs", "crates/cclab-vortex/src/render/camera.rs"]
  - id: vortex-text-renderer
    title: "Bitmap Text Rendering"
    depends: [vortex-render-layers]
    context_refs:
      codebase: ["crates/cclab-vortex/src/render/sprite.rs"]
      spec: ["vortex-render-wgpu"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 6 }
    affected_code: ["crates/cclab-vortex/src/render/text.rs"]
  - id: vortex-game-state
    title: "Global Game State Machine"
    depends: [vortex-event-bus]
    context_refs:
      codebase: ["crates/cclab-vortex/src/core/app.rs"]
      spec: ["vortex-core-architecture"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 7 }
    affected_code: ["crates/cclab-vortex/src/core/state.rs"]
  - id: vortex-input-interaction
    title: "Input Mapping and Interaction"
    depends: [vortex-event-bus, vortex-render-layers]
    context_refs:
      codebase: ["crates/cclab-vortex/src/core/input.rs"]
      spec: ["vortex-ecs-engine"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 4 }
    affected_code: ["crates/cclab-vortex/src/core/input.rs"]
history:
  - timestamp: 2026-02-14T10:28:12.958875+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: vortex-p1-batch

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((vortex-p1-batch))  
    Engine Core
      Event Bus
      Game State Machine
    Rendering
      Layered Rendering
      Camera System
      Bitmap Text
    Input
      Input Mapping
      Player Interaction
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  vortex_event_bus["vortex-event-bus\n codebase: crates/cclab-vortex/src/core/app.rs"]
  vortex_render_layers["vortex-render-layers\n codebase: crates/cclab-vortex/src/render/sprite.rs\n gaps: codebase_spec#3"]
  vortex_text_renderer["vortex-text-renderer\n codebase: crates/cclab-vortex/src/render/sprite.rs\n gaps: codebase_spec#6"]
  vortex_game_state["vortex-game-state\n codebase: crates/cclab-vortex/src/core/app.rs\n gaps: codebase_spec#7"]
  vortex_input_interaction["vortex-input-interaction\n codebase: crates/cclab-vortex/src/core/input.rs\n gaps: codebase_spec#4"]

  vortex_event_bus --> vortex_render_layers
  vortex_render_layers --> vortex_text_renderer
  vortex_event_bus --> vortex_game_state
  vortex_event_bus --> vortex_input_interaction
  vortex_render_layers --> vortex_input_interaction
```

## Spec Execution Order

1. **vortex-event-bus** — Internal Event Bus System
   - code: crates/cclab-vortex/src/core/event.rs
2. **vortex-game-state** — Global Game State Machine
   - depends: vortex-event-bus
   - code: crates/cclab-vortex/src/core/state.rs
3. **vortex-render-layers** — Layered Rendering and Camera Integration
   - depends: vortex-event-bus
   - code: crates/cclab-vortex/src/render/layers.rs, crates/cclab-vortex/src/render/camera.rs
4. **vortex-input-interaction** — Input Mapping and Interaction
   - depends: vortex-event-bus, vortex-render-layers
   - code: crates/cclab-vortex/src/core/input.rs
5. **vortex-text-renderer** — Bitmap Text Rendering
   - depends: vortex-render-layers
   - code: crates/cclab-vortex/src/render/text.rs

</proposal>
