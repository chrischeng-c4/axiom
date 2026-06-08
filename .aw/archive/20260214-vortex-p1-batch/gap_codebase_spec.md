---
change_id: vortex-p1-batch
type: gap_codebase_spec
created_at: 2026-02-14T10:19:20.263268+00:00
updated_at: 2026-02-14T10:19:20.263268+00:00
---

# Gap Analysis: Codebase vs Spec

## Specs with No Matching Implementation

- **vortex-agent-bt [High]**: The specification defines Behavior Trees and Nova agent synchronization, but the codebase context does not include any implementation for the `agent/` module or behavior tree logic.
- **Parallel Systems (vortex-ecs-engine R3) [Medium]**: The ECS specification requires parallel system execution support. However, `ecs/world.rs` only implements basic entity/component management without a scheduler or parallel dispatch capability.
- **Resource Management (vortex-td-mechanics R4) [Medium]**: The TD mechanics specification includes resource management, but the current implementations in `tower.rs`, `enemy.rs`, and `wave.rs` lack any logic for currency, costs, or player resources.
- **Camera System (vortex-render-wgpu R3) [Medium]**: While a `camera.rs` file exists in the repository, it was not analyzed in the codebase context, and the `render_batch` logic in `sprite.rs` currently uses a hardcoded identity matrix instead of an integrated camera system.

## Code with No Matching Spec

- **crates/cclab-vortex/src/core/input.rs [Medium]**: Input handling code for keyboard and mouse exists, but the spec context explicitly identifies "Detailed input mapping" as a gap, indicating that the existing code lacks a formal specification for action mapping.
- **crates/cclab-vortex/src/render/tilemap.rs [Medium]**: A tilemap rendering implementation exists in the codebase, but there is no corresponding specification covering tile-based rendering or map data structures in the spec context.
- **crates/cclab-vortex/src/render/ui.rs [Medium]**: UI rendering code exists, but high-level UI specifications (such as HUD or menu systems) are missing from the scanned spec groups.
- **crates/cclab-vortex/src/agent/state_machine.rs [Low]**: A generic state machine implementation exists in the `agent/` module, but the spec context identifies "Global Game State Machine" as a gap, leaving this utility undocumented in the architectural specs.
