---
change_id: vortex-engine
type: gap_codebase_knowledge
created_at: 2026-02-14T06:36:25.373271+00:00
updated_at: 2026-02-14T06:36:25.373271+00:00
---

## Convention Violations
- **MCP Tool Loading Strategy**: `crates/cclab-server/src/mcp/router.rs` implements a static `UnifiedMcpRouter`. This deviates from the dynamic configuration patterns documented in `40-mcp/index.md`, which specify stage-specific tool loading to minimize tool surface area. (Severity: Medium)
- **Vortex Directory Structure**: The subdirectories in `crates/cclab-vortex/` (`render`, `ecs`, `td`) do not align with the "System Archetypes" or "Spec Catalog" defined in `spec-to-code/spec-model.md`. (Severity: Low)

## Pattern Mismatches
- **GIL Management**: `crates/cclab-orbit/src/loop_impl.rs` lacks evidence of the explicit GIL release strategies required by the performance standards in `orbit/performance-tuning.md` for multi-language event loops. (Severity: High)
- **Spec-to-Code Traceability**: The implementation of the `Agent` trait in `crates/cclab-nova/src/agents/mod.rs` and the tool registration in `UnifiedMcpRouter` do not manifest the "Spec-to-Code Mapping" pattern from `spec-to-code/spec-model.md`, specifically regarding mapping to core spec types. (Severity: Medium)

## Alignment Gaps
- **Data Persistence Pattern**: The planned modules for `crates/cclab-vortex/` do not incorporate the "Data Mapper Pattern" required for high-complexity systems in `05-titan/architecture-guide.md`. (Severity: Medium)
- **Tool Surface Area**: `UnifiedMcpRouter` aggregates multiple toolsets (Genesis, Prism, Aurora) statically, which exceeds the tool exposure limits recommended in `40-mcp/claude-mcp.md` to prevent cognitive load issues. (Severity: Medium)