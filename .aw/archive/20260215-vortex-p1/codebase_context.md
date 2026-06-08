---
change_id: vortex-p1
type: codebase_context
created_at: 2026-02-14T17:25:36.315586+00:00
updated_at: 2026-02-14T17:25:36.315586+00:00
iteration: 4
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
---

# Codebase Context

## Analyzed Files

- **crates/cclab-vortex/src/lib.rs** — Crate root, exports 6 modules: ecs, agent, core, render, td, mcp
  - symbols: `ecs`, `agent`, `core`, `render`, `td`, `mcp`
- **crates/cclab-vortex/src/core/event.rs** — Sync-only typed event queue using VecDeque, type-erased storage
  - symbols: `EventBus`, `Event`, `TypedEventQueue`, `ErasedEventQueue`, `WindowResizedEvent`, `StateTransitionEvent`, `EntitySpawnedEvent`, `EntityDespawnedEvent`
- **crates/cclab-vortex/src/core/state.rs** — Game phase enum with 5 variants and transition-guarded state machine
  - symbols: `GamePhase`, `GameStateMachine`
- **crates/cclab-vortex/src/core/app.rs** — App entry point implementing winit ApplicationHandler, holds World, Schedule, window, render_ctx
  - symbols: `App`, `AppConfig`
- **crates/cclab-vortex/src/core/input.rs** — Input state tracking (keyboard/mouse)
  - symbols: `Input`
- **crates/cclab-vortex/src/core/time.rs** — Frame time and delta tracking
  - symbols: `Time`
- **crates/cclab-vortex/src/core/math.rs** — Math types and re-exports (Vec2, Rect)
  - symbols: `Vec2`, `Rect`
- **crates/cclab-vortex/src/core/mod.rs** — Core module re-exports
- **crates/cclab-vortex/src/agent/behavior_tree.rs** — Behavior tree with composite/decorator/action/condition nodes
  - symbols: `BtNode`, `BtAction`, `BtCondition`, `BtContext`, `BtStatus`, `DecoratorType`
- **crates/cclab-vortex/src/agent/state_machine.rs** — Generic FSM with event-driven transitions and on_enter/on_exit callbacks
  - symbols: `StateMachine`, `StateCallback`
- **crates/cclab-vortex/src/agent/blackboard.rs** — Key-value store for BT context sharing between nodes
  - symbols: `Blackboard`
- **crates/cclab-vortex/src/agent/decision.rs** — Agent decision-making utilities
- **crates/cclab-vortex/src/agent/mod.rs** — Agent module re-exports
- **crates/cclab-vortex/src/mcp/tools.rs** — MCP tool enum definitions and JSON schema generation
  - symbols: `VortexTool`, `ToolSchema`, `tool_schemas`
- **crates/cclab-vortex/src/mcp/state_reader.rs** — Read-only game state accessor for MCP tool handlers
- **crates/cclab-vortex/src/mcp/mod.rs** — MCP module re-exports
- **crates/cclab-vortex/src/render/context.rs** — wgpu device/queue/surface initialization and frame presentation
  - symbols: `RenderContext`
- **crates/cclab-vortex/src/render/sprite.rs** — Sprite vertex layout, instance data, and wgpu render pipeline for batched sprite drawing
  - symbols: `SpriteVertex`, `SpriteInstance`, `SpriteRenderer`, `SPRITE_SHADER`
- **crates/cclab-vortex/src/render/layers.rs** — 5-layer render ordering (Background/World/Effects/Ui/Debug) with sorted sprite queue
  - symbols: `RenderLayer`, `LayeredSprite`, `RenderQueue`, `colored_quad`
- **crates/cclab-vortex/src/render/text.rs** — Bitmap font rendering via sprite instances, ASCII character UV mapping
  - symbols: `BitmapFont`, `TextRenderer`
- **crates/cclab-vortex/src/render/ui.rs** — UI element rendering
- **crates/cclab-vortex/src/render/camera.rs** — Camera view/projection matrix
- **crates/cclab-vortex/src/render/tilemap.rs** — Tilemap rendering
- **crates/cclab-vortex/src/render/mod.rs** — Render module re-exports
- **crates/cclab-vortex/src/td/tower.rs** — Tower placement and targeting logic
- **crates/cclab-vortex/src/td/wave.rs** — Wave spawning and progression
- **crates/cclab-vortex/src/td/components.rs** — Tower defense ECS components
- **crates/cclab-vortex/src/td/economy.rs** — Gold/currency management
- **crates/cclab-vortex/src/td/enemy.rs** — Enemy spawning, movement, and health
- **crates/cclab-vortex/src/td/map.rs** — Map grid and path data
- **crates/cclab-vortex/src/td/mod.rs** — TD module re-exports
- **crates/cclab-vortex/src/ecs/world.rs** — Central ECS data store with entity/component/resource management
  - symbols: `World`
- **crates/cclab-vortex/src/ecs/system.rs** — System trait and Schedule for ordered system execution
  - symbols: `System`, `Schedule`
- **crates/cclab-vortex/src/ecs/entity.rs** — Entity ID type and generation tracking
  - symbols: `Entity`
- **crates/cclab-vortex/src/ecs/query.rs** — ECS query interface
- **crates/cclab-vortex/src/ecs/component.rs** — Component storage traits
- **crates/cclab-vortex/src/ecs/mod.rs** — ECS module re-exports
- **crates/cclab-server/src/mcp/router.rs** — Unified MCP router dispatching to genesis, prism, and aurora tool handlers
  - symbols: `UnifiedMcpRouter`, `list_tools`, `call_tool`, `call_genesis_tool`, `call_prism_tool`, `call_aurora_tool`
- **crates/cclab-server/src/registry.rs** — Project registry for multi-project MCP server
  - symbols: `Registry`
- **crates/cclab-server/src/prism_pool.rs** — Prism handler pool for concurrent analysis requests
  - symbols: `PrismHandlerPool`

## Prism Results

- **prism_symbols** (query: `crates/cclab-vortex/src/lib.rs`)
  - 6 modules: ecs, agent, core, render, td, mcp
- **prism_symbols** (query: `crates/cclab-vortex/src/core/event.rs`)
  - EventBus with TypedEventQueue<T>, ErasedEventQueue trait, Event trait. Methods: publish, read, drain, has_events, flush.
- **prism_symbols** (query: `crates/cclab-vortex/src/core/state.rs`)
  - GamePhase enum (Loading, Menu, Playing, Paused, GameOver). GameStateMachine with transition map.
- **prism_symbols** (query: `crates/cclab-vortex/src/core/app.rs`)
  - App struct with World, Schedule, AppConfig, window, render_ctx. Implements winit ApplicationHandler.
- **prism_symbols** (query: `crates/cclab-vortex/src/mcp/tools.rs`)
  - VortexTool enum with 8 variants. ToolSchema struct. tool_schemas() function.
- **prism_symbols** (query: `crates/cclab-vortex/src/agent/behavior_tree.rs`)
  - BtStatus, BtContext, BtAction trait, BtCondition trait, DecoratorType, BtNode enum with tick().
- **prism_symbols** (query: `crates/cclab-vortex/src/agent/state_machine.rs`)
  - Generic StateMachine<S> with event-driven transitions and on_enter/on_exit callbacks.
- **prism_symbols** (query: `crates/cclab-server/src/mcp/router.rs`)
  - UnifiedMcpRouter dispatching to genesis, prism, and aurora tool handlers.
- **prism_symbols** (query: `crates/cclab-vortex/src/render/text.rs`)
  - BitmapFont, TextRenderer with draw_text, measure_text, draw_number. Imports glam::Vec2 and SpriteInstance.
- **prism_symbols** (query: `crates/cclab-vortex/src/render/sprite.rs`)
  - SpriteVertex, SpriteInstance, SpriteRenderer. Imports RenderContext and glam types.
- **prism_symbols** (query: `crates/cclab-vortex/src/render/layers.rs`)
  - RenderLayer enum (5 layers), LayeredSprite, RenderQueue. Imports glam::Vec2 and SpriteInstance.

## Dependency Graph

- core/app.rs -> ecs/world.rs (World)
- core/app.rs -> ecs/system.rs (Schedule)
- core/app.rs -> render/context.rs (RenderContext)
- core/app.rs -> core/input.rs (Input)
- core/app.rs -> core/time.rs (Time)
- core/app.rs -> core/event.rs (EventBus)
- core/event.rs -> ecs/entity.rs (Entity)
- agent/behavior_tree.rs -> agent/blackboard.rs (Blackboard)
- agent/behavior_tree.rs -> ecs/world.rs (World)
- agent/behavior_tree.rs -> ecs/entity.rs (Entity)
- agent/state_machine.rs -> ecs/world.rs (World)
- agent/state_machine.rs -> ecs/entity.rs (Entity)
- render/sprite.rs -> render/context.rs (RenderContext)
- render/sprite.rs -> [external] glam (Vec2, Mat4)
- render/sprite.rs -> [external] wgpu (Device, RenderPipeline, Buffer)
- render/layers.rs -> render/sprite.rs (SpriteInstance)
- render/layers.rs -> [external] glam (Vec2)
- render/text.rs -> render/sprite.rs (SpriteInstance)
- render/text.rs -> [external] glam (Vec2)
- mcp/tools.rs -> [external] serde_json (Value, json!)
- cclab-server/mcp/router.rs -> cclab-server/registry.rs (Registry)
- cclab-server/mcp/router.rs -> cclab-server/prism_pool.rs (PrismHandlerPool)
- cclab-server/mcp/router.rs -> [external] cclab-genesis (GenesisToolRegistry)
- cclab-server/mcp/router.rs -> [external] cclab-prism (PrismRequest)
- cclab-server/mcp/router.rs -> [external] cclab-aurora (aurora_call_tool, is_aurora_tool)
