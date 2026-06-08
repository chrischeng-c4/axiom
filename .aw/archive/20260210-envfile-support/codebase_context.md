---
change_id: envfile-support
type: codebase_context
created_at: 2026-02-10T02:29:07.607583+00:00
updated_at: 2026-02-10T02:29:07.607583+00:00
iteration: 1
complexity: medium
stage: codebase
prism_tools_used:
  - list_directory
  - read_file
  - search_file_content
---

# Codebase Context

## Analyzed Files

- **crates/cclab-genesis/src/models/change.rs** — Configuration data models (GenesisConfig, WorkflowConfig, etc.) and loading logic.
  - symbols: `GenesisConfig`, `WorkflowConfig`, `AgentsConfig`, `GeminiConfig`, `CodexConfig`, `ClaudeConfig`, `GenesisConfig::load_validated`
- **crates/cclab-genesis/src/mcp/tools/agent.rs** — Implementation of genesis_agent MCP tool, which executes agents via ScriptRunner.
  - symbols: `execute_streaming`, `build_provider_args`
- **crates/cclab-genesis/src/orchestrator/script_runner.rs** — Low-level CLI execution with environment variable support.
  - symbols: `ScriptRunner`, `ScriptRunner::run_llm_raw_streaming`
- **crates/cclab-genesis/src/orchestrator/cli_mapper.rs** — Mapping of common LLM arguments to provider-specific CLI flags.
  - symbols: `LlmProvider`, `LlmArg`

## Dependency Graph

- cclab-genesis/src/mcp/tools/agent.rs -> cclab-genesis/src/models/change.rs (GenesisConfig)
- cclab-genesis/src/mcp/tools/agent.rs -> cclab-genesis/src/orchestrator/script_runner.rs (ScriptRunner)
