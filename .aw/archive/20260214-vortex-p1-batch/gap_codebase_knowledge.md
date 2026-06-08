---
change_id: vortex-p1-batch
type: gap_codebase_knowledge
created_at: 2026-02-14T10:24:39.436719+00:00
updated_at: 2026-02-14T10:24:39.436719+00:00
---

# Gap Analysis: Codebase vs Knowledge (vortex-p1-batch)

## Convention Violations

### 1. Missing Vortex Architecture Documentation
- **File(s)**: `crates/cclab-vortex/`
- **Reference**: `cclab/knowledge/spec-to-code/spec-model.md` (System Archetypes), `cclab/README.md` (Directory Structure)
- **Gap**: The `cclab-vortex` crate is a high-complexity project (ECS + wgpu), but it has no presence in the knowledge base or specs directory. Per `spec-model.md`, a "Full module" archetype should have all six spec types.
- **Severity**: HIGH

### 2. MCP Tool Registry Mismatch
- **File(s)**: `crates/cclab-vortex/src/mcp/tools.rs`
- **Reference**: `cclab/knowledge/40-mcp/dynamic-config.md` (Step 1: Implement Tool Filtering in MCP Server)
- **Gap**: The internal `VortexTool` enum defines 8 tools, but the `tool_schemas` function only exports 5 of them. This violates the centralized tool definition pattern established in the dynamic config strategy.
- **Severity**: LOW

### 3. MCP Routing Isolation
- **File(s)**: `crates/cclab-vortex/src/mcp/tools.rs`
- **Reference**: `cclab/knowledge/40-mcp/index.md` (Global HTTP MCP server with multi-project support)
- **Gap**: Vortex tools are currently isolated in their own crate and not registered in the unified `cclab-server` router. Established convention (as outlined in the MCP index overview) requires tools to be part of the global multi-project routing strategy.
- **Severity**: MEDIUM

## Pattern Mismatches

### 4. Dynamic MCP Configuration Violation
- **File(s)**: `crates/cclab-vortex/src/mcp/tools.rs`
- **Reference**: `cclab/knowledge/40-mcp/dynamic-config.md`
- **Gap**: Knowledge base mandates stage-specific dynamic tool loading to reduce cognitive load. Vortex tools are exported as a monolithic list with no stage metadata or filtering support.
- **Severity**: MEDIUM

### 5. Performance Pattern Mismatch (Allocation Pressure)
- **File(s)**: `crates/cclab-vortex/src/td/tower.rs`, `crates/cclab-vortex/src/td/enemy.rs`
- **Reference**: `cclab/knowledge/orbit/performance-tuning.md` (Pre-allocate buffers)
- **Gap**: The performance guide recommends pre-allocating buffers to reduce allocation pressure. Systems in Vortex use `.collect::<Vec<_>>()` every frame (60 FPS) in the hot game loop, causing significant heap pressure.
- **Severity**: MEDIUM

### 6. Incomplete Render Implementation
- **File(s)**: `crates/cclab-vortex/src/core/app.rs`
- **Reference**: `cclab/knowledge/orbit/performance-tuning.md` (Async Performance Optimization)
- **Gap**: `render_ctx` is correctly initialized in `App::resumed` but is entirely ignored in `WindowEvent::RedrawRequested`. This prevents any actual rendering from occurring, violating the functional purpose of the engine.
- **Severity**: HIGH
