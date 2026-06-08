---
change: agent-pyo3
group: agent-pyo3-bindings
date: 2026-03-18
---

# Requirements

Create `crates/cclab-agent-pyo3/` — a new maturin-built crate exposing all layers of `cclab-agent` to Python under the module name `_agent` (Python import: `cclab_agent`).

**Engine layer** (wrap with `Py{Name}` convention):
- `PyClaudeProvider`, `PyOpenAIProvider`, `PyGeminiProvider`
- `PyToolRegistry`, `PyToolExecutor` (Python callable → Rust Tool bridge)
- `PyContextManager`
- `complete_structured()` async binding
- `PyMessage`, `PyRole`, `PyToolCall`, `PyTokenUsage`

**Integrations layer**:
- `PyGitHubIntegration`, `PyGitLabIntegration`, `PyJiraIntegration`

**Agents layer**:
- `PyRestructureAgent`, `PySpecAgent`, `PyCodeAgent`, `PyReviewAgent`
- `PyCRRCycle`

**Conventions**:
- Wrapper types: `Py{OriginalName}`
- Async methods: `pyo3_async_runtimes::tokio::future_into_py`
- Python stubs: `__init__.py` + `__init__.pyi` via `cclab lens gen-stub`
- Build: `maturin develop`; install in Conductor via `uv add --path /path/to/cclab-agent-pyo3`

**Acceptance criteria**:
- `from cclab_agent import ClaudeProvider, OpenAIProvider, GeminiProvider` works
- `from cclab_agent import SpecAgent, ReviewAgent, CRRCycle` works
- `from cclab_agent import GitLabIntegration` works
- Full Python stubs with type hints are generated
