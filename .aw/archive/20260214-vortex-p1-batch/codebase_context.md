---
change_id: vortex-p1-batch
type: codebase_context
created_at: 2026-02-14T10:15:47.576420+00:00
updated_at: 2026-02-14T10:15:47.576420+00:00
iteration: 2
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - grep_search
  - glob
---

# Codebase Context

## Analyzed Files

- **crates/cclab-vortex/src/core/app.rs** — Main entry point and game loop driver using winit.
  - symbols: `App`, `AppConfig`, `ApplicationHandler for App`
- **crates/cclab-vortex/src/render/sprite.rs** — Batched sprite rendering logic (currently solid colors only).
  - symbols: `SpriteRenderer`, `SpriteVertex`, `SpriteInstance`, `render_batch`
- **crates/cclab-vortex/src/ecs/world.rs** — Central ECS container for entities, components, and resources.
  - symbols: `World`, `spawn`, `despawn`, `insert_resource`, `get_resource_mut`
- **crates/cclab-vortex/src/render/context.rs** — GPU context manager wrapping wgpu state.
  - symbols: `RenderContext`
- **crates/cclab-vortex/src/core/input.rs** — Input state tracking resource.
  - symbols: `Input`
- **crates/cclab-vortex/src/core/time.rs** — Time management resource.
  - symbols: `Time`
- **crates/cclab-vortex/src/td/tower.rs** — Tower targeting and projectile systems.
  - symbols: `tower_targeting_system`, `projectile_system`
- **crates/cclab-vortex/src/td/enemy.rs** — Enemy movement and death systems.
  - symbols: `enemy_movement_system`, `enemy_death_system`
- **crates/cclab-vortex/src/td/wave.rs** — Wave configuration and spawning system.
  - symbols: `WaveSystem`, `WaveConfig`, `SpawnGroup`, `wave_spawn_system`

## Prism Results

- **prism_symbols** (query: `prism_symbols(crates/cclab-vortex/src/td/tower.rs)`)
  - Found tower_targeting_system and projectile_system functions taking &World.
- **prism_symbols** (query: `prism_symbols(crates/cclab-vortex/src/td/enemy.rs)`)
  - Found enemy_movement_system and enemy_death_system functions taking &World.
- **prism_symbols** (query: `prism_symbols(crates/cclab-vortex/src/td/wave.rs)`)
  - Found WaveSystem struct and wave_spawn_system function taking &World.
- **grep_search** (query: `grep_search(render_ctx usage)`)
  - render_ctx is initialized in App::resumed but never called for rendering in RedrawRequested.

## Dependency Graph

- crates/cclab-vortex/src/core/app.rs -> crates/cclab-vortex/src/ecs/world.rs (World management)
- crates/cclab-vortex/src/core/app.rs -> crates/cclab-vortex/src/render/context.rs (RenderContext initialization)
- crates/cclab-vortex/src/render/sprite.rs -> crates/cclab-vortex/src/render/context.rs (wgpu device/queue usage)
- crates/cclab-vortex/src/td/tower.rs -> crates/cclab-vortex/src/ecs/world.rs (System logic on World)
- crates/cclab-vortex/src/td/enemy.rs -> crates/cclab-vortex/src/ecs/world.rs (System logic on World)
- crates/cclab-vortex/src/td/wave.rs -> crates/cclab-vortex/src/ecs/world.rs (System logic on World)
