---
change_id: vortex-p1
type: gap_codebase_knowledge
created_at: 2026-02-14T17:29:19.903222+00:00
updated_at: 2026-02-14T17:29:19.903222+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention violations

### MEDIUM severity

1. **MCP tool registration pattern not followed** — Knowledge doc 40-mcp/http-server.md documents MCP tool registration via HTTP server with X-Genesis-Project header and JSON-RPC 2.0. VortexTool (mcp/tools.rs) defines tool schemas but UnifiedMcpRouter (cclab-server/mcp/router.rs) has no call_vortex_tool dispatch path. The existing genesis/prism/aurora routing pattern is not replicated for vortex.

2. **CLI module not registered** — Knowledge doc (CLAUDE.md CLI Auto-Registration section) documents linkme distributed_slice pattern for CLI subcommand registration. IonCli is the existing example. cclab-vortex has no CliModule implementation or #[distributed_slice(CLI_MODULES)] registration.

### LOW severity

3. **File size awareness** — Knowledge pitfall documents files >= 500 lines as split candidates, >= 1000 lines as mandatory splits. Current vortex files are within limits, but P1 scope adds significant new code across event bus, state machine, AI, rendering, input, debug overlay, tests, and MCP integration. No evidence this constraint is tracked during implementation planning.

## Pattern mismatches

### MEDIUM severity

4. **EventBus lacks async pattern** — Knowledge doc orbit/bridge-internals.md documents async event bridge pattern for cross-boundary communication. EventBus (core/event.rs) is sync-only using VecDeque. The Orbit async pattern is documented as relevant but not reflected in current implementation.

5. **State machine archetype mismatch** — Knowledge doc spec-to-code/spec-model.md documents workflow/state machine archetype (API Spec + State+ + Sequence+ + Flowchart+ + Requirement+). GameStateMachine (core/state.rs) implements a simple transition map without formal state machine spec backing.

### LOW severity

6. **Stack overflow risk unmitigated** — Knowledge pitfall documents tokio worker thread stack overflow (#182) causing silent crashes. EventBus is sync-only and does not interact with tokio currently, but async extension (#345) and MCP server integration (#330) will. No evidence of stack size consideration in current codebase."