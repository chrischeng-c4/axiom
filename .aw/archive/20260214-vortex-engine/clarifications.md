---
change: vortex-engine
date: 2026-02-14
---

# Clarifications

## Q1: Scope
- **Question**: Should we implement all phases in one change, or focus on a specific subset first?
- **Answer**: All 6 phases — implement the full crate: ECS, Agent, Core, Render, TD, MCP in one change.
- **Rationale**: User wants the complete crate delivered as a single cohesive change encompassing all core engine subsystems and gameplay logic.

## Q2: Renderer
- **Question**: For the wgpu renderer, should we include full GPU rendering or use a simpler headless approach first?
- **Answer**: Full wgpu — real GPU-based 2D rendering with sprites, tilemaps, camera as planned.
- **Rationale**: User wants the full rendering pipeline from the start, not a stub.

## Q3: Git Workflow
- **Question**: Which git workflow should we use?
- **Answer**: in_place — work on the current branch (feat/vortex-engine).
- **Rationale**: Branch already exists and is named for this feature.

