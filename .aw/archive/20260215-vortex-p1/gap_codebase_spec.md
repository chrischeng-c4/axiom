---
change_id: vortex-p1
type: gap_codebase_spec
created_at: 2026-02-14T17:28:23.482973+00:00
updated_at: 2026-02-14T17:28:23.482973+00:00
---

# Gap Analysis: Codebase vs Spec

## Code without matching spec

### HIGH severity

1. **EventBus (core/event.rs)** — Sync-only implementation with publish/read/drain/flush exists. No dedicated spec covers EventBus; vortex-core-architecture covers engine lifecycle but not event system.

2. **MCP tools (mcp/tools.rs, mcp/state_reader.rs) + server routing (cclab-server/mcp/router.rs)** — VortexTool enum defines 8 tool variants with schemas. UnifiedMcpRouter dispatches to genesis/prism/aurora but has no vortex routing path. No spec covers this integration.

3. **Player input (core/input.rs)** — Input struct tracks keyboard/mouse state. No spec covers mouse-based tower placement interaction.

### MEDIUM severity

4. **GameStateMachine (core/state.rs)** — GamePhase has 5 variants. Spec vortex-core-architecture covers engine lifecycle but not game-level state transitions.

5. **Agent FSM (agent/state_machine.rs)** — Generic StateMachine<S> exists. Spec vortex-agent-bt covers only behavior trees, not the FSM component.

6. **Render layers (render/layers.rs)** — RenderLayer enum, LayeredSprite, RenderQueue exist. Spec vortex-render-wgpu covers batch rendering and camera but not Z-ordered layer composition.

7. **Text rendering (render/text.rs)** — BitmapFont and TextRenderer exist. Spec vortex-render-wgpu does not cover text rendering.

## Specs without matching implementation

### HIGH severity

1. **vortex-core-architecture** (spec ID: vortex-core-architecture) — R1 defines VortexEngine Lifecycle Interface Contract. Extended game states (LevelSelect, Victory) per #371 are not implemented in GamePhase enum.

2. **vortex-agent-bt** (spec ID: vortex-agent-bt) — R1 Composite Nodes, R3 Nova Agent Sync. Spec covers BT only; no spec-level pairing with StateMachine<S> for #334 Both requirement.

### MEDIUM severity

3. **vortex-render-wgpu** (spec ID: vortex-render-wgpu) — R1 Batch Rendering, R3 Camera System. Does not specify layered rendering (Z-ordering) or bitmap text rendering, both partially implemented in code.

4. **vortex-td-mechanics** (spec ID: vortex-td-mechanics) — R2 Turret Logic, Wave Lifecycle. No integration test coverage defined for TD gameplay (#343).

## Missing specs (no spec exists)

### HIGH severity

1. **Event bus** — No dedicated spec for #345. EventBus code exists (core/event.rs) but async listener support is not specified or implemented.

2. **Input/interaction** — No spec for #332. Input code exists (core/input.rs) but tower placement UI interaction is not specified.

### MEDIUM severity

3. **Debug overlay** — No spec for #353. FPS/entity count/system timings overlay is not specified or implemented.

4. **MCP server integration** — No spec for #330. VortexTool and state_reader code exist but server router integration is not specified."