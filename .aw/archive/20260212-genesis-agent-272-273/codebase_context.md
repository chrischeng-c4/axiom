---
change_id: genesis-agent-272-273
type: codebase_context
created_at: 2026-02-12T11:28:36.072408+00:00
updated_at: 2026-02-12T11:28:36.072408+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - grep (manual codebase search for genesis_agent references)
---

# Codebase Context

## Analyzed Files

- **crates/cclab-genesis/src/mcp/tools/agent.rs** — PRIMARY: The genesis_agent MCP tool implementation. Must be renamed to genesis_delegate_agent, action enum expanded from 3 to 30+, response changed from raw stdout to artifact-oriented.
  - symbols: `definition()`, `parse_agent_spec()`, `build_provider_args()`, `render_template()`, `Verification`, `get_verification()`, `execute_streaming()`, `LLM_EXPLORE`, `LLM_REVIEW`, `MCP_SERVER_NAME`
- **crates/cclab-genesis/src/mcp/tools/mod.rs** — Tool registry. References 'genesis_agent' in all_tools_vec(), call_tool(), call_tool_streaming(). Must rename to 'genesis_delegate_agent'.
  - symbols: `ToolRegistry`, `all_tools_vec()`, `call_tool()`, `call_tool_streaming()`
- **crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs** — Defines GENESIS_AGENT_TOOL constant used by run_change to build next array. Must rename constant value.
  - symbols: `GENESIS_AGENT_TOOL`
- **crates/cclab-genesis/src/mcp/tools/run_change/mod.rs** — Uses GENESIS_AGENT_TOOL to build next array entries for agent-delegated actions. No code change needed (uses constant).
  - symbols: `action_to_artifact()`, `build_next_array()`
- **crates/cclab-genesis/src/mcp/config.rs** — MCP config generator for Gemini/Codex/Claude. Has hardcoded 'genesis_agent' in excludeTools/disabled_tools for recursion prevention. Must rename.
  - symbols: `ensure_gemini_mcp_config()`, `ensure_codex_mcp_config()`
- **crates/cclab-genesis/src/orchestrator/cli_mapper.rs** — CLI argument builder. Has 'genesis_agent' in DisallowedMcpTools for all 3 providers. Must rename.
  - symbols: `DisallowedMcpTools`
- **.claude/skills/cclab-genesis-agent/SKILL.md** — Claude Code skill definition. Must rename skill, update tool name, expand action list.
- **crates/cclab-genesis/src/mcp/tools/run_change/explore_spec.rs** — Per-action prompt builder for explore_spec. Example of run_change prompt that delegate_agent should use.
- **crates/cclab-genesis/src/mcp/tools/context.rs** — References genesis_agent in response hints (caller='agent' context).
- **crates/cclab-genesis/src/mcp/tools/review.rs** — References genesis_agent in response hints (caller='agent' context).

## Dependency Graph

- agent.rs depends on: orchestrator/cli_mapper.rs (build_provider_args), models/frontmatter.rs (StatePhase, LlmCall), state.rs (StateManager)
- mod.rs depends on: agent.rs (definition, execute_streaming)
- run_change/mod.rs depends on: helpers.rs (GENESIS_AGENT_TOOL constant)
- config.rs references: 'genesis_agent' string literal in excludeTools/disabled_tools
- cli_mapper.rs references: 'genesis_agent' string literal in DisallowedMcpTools
