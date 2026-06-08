---
change_id: vortex-p1
type: gap_spec_knowledge
created_at: 2026-02-14T17:30:21.126289+00:00
updated_at: 2026-02-14T17:30:21.126289+00:00
---

# Gap Analysis: Spec vs Knowledge

## Spec responsibilities contradicting knowledge architecture

### LOW severity

1. **vortex-core-architecture R1 lifecycle vs MCP HTTP server pattern** — Spec defines engine lifecycle as single-threaded game loop (init → run → shutdown). Knowledge 40-mcp/http-server.md documents MCP server as async HTTP daemon on port 3000. No spec addresses how async MCP server coexists with sync game loop thread.

## Knowledge patterns not reflected in any spec

### HIGH severity

1. **MCP Tool Registration pattern (40-mcp/http-server.md)** — Knowledge documents HTTP MCP server with project isolation headers, JSON-RPC 2.0, dynamic registry. No vortex spec references this pattern or defines how vortex MCP tools integrate into the server router.

2. **Distributed Slice CLI Registration (30-claude/skills.md, CLAUDE.md)** — Knowledge documents linkme distributed_slice pattern for CLI module auto-registration. No vortex spec covers CLI subcommand registration.

### MEDIUM severity

3. **Async Event Bridge pattern (orbit/bridge-internals.md)** — Knowledge documents Orbit's async event patterns for cross-boundary communication. Spec vortex-core-architecture covers sync engine lifecycle. No spec addresses async listener support for event bus.

4. **State Machine spec archetype (spec-to-code/spec-model.md)** — Knowledge documents workflow/state machine archetype requiring State+ spec type. Spec vortex-core-architecture does not use State+ for game state machine definition.

5. **File size constraint** — Knowledge documents >= 1000 lines must split, >= 500 consider split. No spec references this constraint or plans module structure to comply with it across the 8 P1 features.

## Responsibility boundary misalignments

### MEDIUM severity

6. **vortex-agent-bt scope vs knowledge ECS pattern** — Spec vortex-agent-bt R3 defines Nova Agent Sync for external LLM integration. Knowledge ECS pattern documents systems as unit of game logic execution. Spec boundary between BT agent system and ECS system execution is unclear — vortex-agent-bt spec does not specify how BT/FSM agents register as ECS systems.

7. **vortex-render-wgpu scope vs knowledge spec-to-code model** — Spec covers batch rendering and camera. Knowledge spec-model.md defines layered spec approach (Sequence+ for module structure, Class+ for architecture). Spec does not decompose rendering into separately specifiable subsystems (layers, text, UI) despite code already having this structure."