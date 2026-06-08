---
change: agent-crr-review
group: agent-crr-core
date: 2026-03-18
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-agent/agents.md | agent-core | high | Agent trait (run, run_with_handler) — ReviewAgent implements this or TypedAgent, Agent builder pattern — CRRCycle and ReviewAgent follow same builder convention, Agent execution loop — CRR wraps multiple agent runs in a loop |
| cclab-agent/llm-providers.md | llm-integration | high | LLMProvider trait — ReviewAgent calls complete() or complete_structured(), CompletionRequest/Response — used for review calls |
| cclab-agent/architecture.md | system-design | medium | Module structure — agents/review/ fits in agents module, Trait hierarchy — CRRCycle orchestrates agents |
| cclab-agent/error-handling.md | error-handling | medium | NovaError — MaxRevisionsExceeded error variant needed |

