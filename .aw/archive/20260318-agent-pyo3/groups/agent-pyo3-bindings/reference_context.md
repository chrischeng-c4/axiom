---
change: agent-pyo3
group: agent-pyo3-bindings
date: 2026-03-18
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-agent/agents.md | agent-core | high | Agent trait — wrap as PyAgent, Builder pattern — PyBuilder with method chaining |
| cclab-agent/llm-providers.md | llm | high | LLMProvider trait — PyClaudeProvider, PyOpenAIProvider, PyGeminiProvider, CompletionRequest/Response — Py* wrappers |
| cclab-agent/architecture.md | system | high | Module structure — pyo3_bindings/ layout |
| cclab-agent/restructure-agent.md | agents | high | RestructureAgent — PyRestructureAgent wrapper |
| cclab-agent/review-agent.md | agents | high | ReviewAgent, CRRCycle — Py* wrappers |

