---
change_id: prompt-registry
type: codebase_context
created_at: 2026-02-08T17:51:33.815261+00:00
updated_at: 2026-02-08T17:51:33.815261+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - manual exploration
  - git log
  - git show
---

# Codebase Context

## Analyzed Files

- **crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs** — Shared helpers + tool name constants (GENESIS_AGENT_TOOL, GENESIS_RUN_CHANGE_TOOL)
  - symbols: `GENESIS_AGENT_TOOL`, `GENESIS_RUN_CHANGE_TOOL`, `phase_to_string`, `extract_review_info`, `count_spec_files`, `analyze_specs`
- **crates/cclab-genesis/src/mcp/tools/run_change/mod.rs** — Entry point, phase routing, add_executor_info
  - symbols: `route`, `add_executor_info`, `action_to_artifact`
- **crates/cclab-genesis/src/mcp/tools/run_change/clarify.rs** — 2 inline prompts (Clarify, ConfirmUnderstanding)
  - symbols: `handle`
- **crates/cclab-genesis/src/mcp/tools/run_change/explore_spec.rs** — 3 inline prompts (ExploreSpec, ReviewSpecContext, ReviseSpecContext)
  - symbols: `handle`
- **crates/cclab-genesis/src/mcp/tools/run_change/explore_knowledge.rs** — 3 inline prompts - nearly identical structure to explore_spec
  - symbols: `handle`
- **crates/cclab-genesis/src/mcp/tools/run_change/explore_codebase.rs** — 3 inline prompts - nearly identical structure to explore_spec
  - symbols: `handle`
- **crates/cclab-genesis/src/mcp/tools/run_change/proposal.rs** — 3 inline prompts (CreateProposal, ReviewProposal, ReviseProposal)
  - symbols: `handle`
- **crates/cclab-genesis/src/mcp/tools/run_change/spec.rs** — 3 inline prompts (CreateSpec, ReviewSpec, ReviseSpec)
  - symbols: `handle`
- **crates/cclab-genesis/src/mcp/tools/run_change/tasks.rs** — 2 inline prompts (GenerateTasks, Finish)
  - symbols: `handle`
- **crates/cclab-genesis/src/mcp/tools/run_change/implement.rs** — 5 inline prompts (Begin/Resume/Review/Resolve/Complete)
  - symbols: `handle`
- **crates/cclab-genesis/src/mcp/tools/run_change/merge.rs** — 5 inline prompts (Begin/Resume/Review/Fix/Complete)
  - symbols: `handle`
- **crates/cclab-genesis/src/mcp/tools/agent.rs** — genesis_agent MCP tool - uses ScriptRunner::run_llm_raw directly (lacks quota detection, timeout, fallback)
  - symbols: `execute`, `parse_agent_spec`, `build_provider_args`, `render_template`
- **crates/cclab-genesis/src/orchestrator/script_runner.rs** — Low-level subprocess executor - run_llm / run_llm_raw / run_llm_with_cwd
  - symbols: `ScriptRunner`, `run_llm`, `run_llm_raw`, `run_llm_with_cwd`, `run_command_raw`
- **crates/cclab-genesis/src/orchestrator/mod.rs** — Orchestrator module - currently only exports ScriptRunner + ModelSelector + LlmProvider
  - symbols: `ScriptRunner`, `ModelSelector`, `LlmProvider`

## Prism Results

- **git log/show** (query: `Deleted orchestrator files in 210c44f`)
  - commit 210c44f deleted 6 orchestrator modules: agent_runner.rs (352L), claude.rs (397L), codex.rs (596L), gemini.rs (827L), prompts.rs (1251L), quota_detection.rs (239L). These provided AgentRunner with quota detection + auto-fallback, per-artifact agent config, and per-provider orchestrators. Deleted because run_change was refactored to 'only return prompts, never execute agents'. But genesis_agent now needs this functionality.
- **git show** (query: `AgentRunner::run_for_artifact pattern`)
  - AgentRunner.run_for_artifact(WorkflowArtifact, operation_closure) reads agent list from config, tries each agent in order, detects quota errors via check_quota_error(), falls back with 5s delay, returns AgentRunResult with output + failed_agents list. This is exactly what genesis_agent needs instead of raw ScriptRunner.

## Dependency Graph

- agent.rs -> ScriptRunner::run_llm_raw (should use AgentRunner instead)
- AgentRunner -> GeminiOrchestrator / CodexOrchestrator / ClaudeOrchestrator
- AgentRunner -> quota_detection::check_quota_error
- {Provider}Orchestrator -> ScriptRunner (low-level)
- run_change flow files -> helpers.rs (prompt constants)
- mod.rs add_executor_info -> workflow_common::get_executor_chain (config-based agent list)
