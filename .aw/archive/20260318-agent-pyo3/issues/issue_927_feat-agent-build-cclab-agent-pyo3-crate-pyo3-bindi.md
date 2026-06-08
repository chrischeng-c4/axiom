---
number: 927
title: "feat(agent): Build cclab-agent-pyo3 crate — PyO3 bindings for Python"
state: open
labels: [enhancement, crate:agent, P0]
group: "agent-pyo3-bindings"
---

# #927 — feat(agent): Build cclab-agent-pyo3 crate — PyO3 bindings for Python

**Parent**: #920

## Summary

PyO3 bindings exposing cclab-agent to Python consumers (Conductor).

## Expose

### Engine
- PyClaudeProvider, PyOpenAIProvider, PyGeminiProvider
- PyToolRegistry, PyToolExecutor (Python callable → Rust Tool bridge)
- PyContextManager
- complete_structured() binding
- PyMessage, PyRole, PyToolCall, PyTokenUsage

### Integrations
- PyGitHubIntegration, PyGitLabIntegration, PyJiraIntegration

### Agents
- PyRestructureAgent
- PySpecAgent, PyCodeAgent, PyReviewAgent
- PyCRRCycle

## Convention
- Wrapper types: Py{OriginalName}
- Module name: _agent
- Async: pyo3_async_runtimes::tokio::future_into_py
- Stubs: __init__.py + __init__.pyi via cclab lens gen-stub

## Install
```bash
cd crates/cclab-agent-pyo3 && maturin develop
# In Conductor:
uv add --path /path/to/cclab-agent-pyo3
```

## Dependencies
- All other cclab-agent issues (this wraps everything)

## Test Plan
- [ ] from cclab_agent import ClaudeProvider, OpenAIProvider, GeminiProvider
- [ ] from cclab_agent import SpecAgent, ReviewAgent, CRRCycle
- [ ] from cclab_agent import GitLabIntegration
- [ ] Python stubs with full type hints
