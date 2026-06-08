---
change: reference-context-agent
group: reference-context-agent
date: 2026-03-18
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| reference-context.md | cclab-sdd/logic | high | artifact schema (spec_id, spec_group, relevance, key_requirements), CRR cycle (max_revisions=1, auto-approve), phase transition (PreClarificationsCreated -> PostClarificationsCreated) |
| agents.md | cclab-agent | high | Agent trait (run, run_with_handler), Agent execution loop (turn management, tool calls), AnalystAgentBuilder (builder pattern) |
| architecture.md | cclab-agent | high | Module dependencies (agents -> context, tools, llm), C4 system context diagram, Agent class diagram relationships |
| restructure-agent.md | cclab-agent | high | Stateless agent pattern, typed run(input: &RestructureInput) -> NovaResult<RestructureOutput>, using complete_structured() for LLM calls, Builder pattern (RestructureAgentBuilder), SpecStore.search(query) -> Vec<SpecExcerpt> |
| review-agent.md | cclab-agent | high | CRRCycle pattern (creator, reviewer, reviser), ReviewAgent for quality review |
| llm-providers.md | cclab-agent | medium | LLMProvider trait (complete), CompletionRequest schema (messages, model, temperature, max_tokens), CompletionResponse schema |
| core-types.md | cclab-agent | high | Role enum (system, user, assistant, tool), Message schema (role, content, tool_calls), ToolCall/ToolResult schemas |
| tools.md | cclab-agent | medium | Tool trait (name, description, execute), ToolRegistry (register, get), ToolExecutor (timeout, retries) |
| error-handling.md | cclab-agent | medium | NovaError variants (LLMError, ToolError, Config), Error classification (is_retriable, requires_user_action) |
| tools-analysis.md | cclab-agent | medium | AskUserTool (for clarification), RecordFindingTool, TakeNoteTool |
| context.md | cclab-agent | low | Background knowledge about the crate, not a direct dependency. |
| README.md | cclab-agent | low | Overall project scope and spec mapping |

