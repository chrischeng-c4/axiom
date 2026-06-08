---
change: restructure-agent
group: restructure-agent-core
date: 2026-03-18
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-agent/agents.md | agent-core | high | Agent trait interface (run, run_with_handler) — base for new TypedAgent<I, O>, Agent builder pattern (CodingAgentBuilder) — RestructureAgentBuilder follows same pattern, Agent config structure (system_prompt, model, max_turns) |
| cclab-agent/llm-providers.md | llm-integration | high | LLMProvider trait — RestructureAgent calls complete() via complete_structured(), CompletionRequest schema (messages, model, response_schema), CompletionResponse schema (content, usage) |
| cclab-agent/architecture.md | system-design | high | Module dependency graph — agents/restructure/ fits in agents module, Class hierarchy — TypedAgent<I, O> extends Agent abstraction |
| cclab-agent/core-types.md | type-system | medium | Message type — used in prompt assembly, Role enum — system/user/assistant for prompt construction |
| cclab-agent/error-handling.md | error-handling | medium | NovaError enum — RestructureAgent errors (LLMError, SchemaValidationError) |

