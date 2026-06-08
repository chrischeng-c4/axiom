---
id: improve-nova-maturity
type: proposal
version: 1
created_at: 2026-01-28T08:39:36.751213+00:00
updated_at: 2026-01-28T08:39:36.751213+00:00
author: mcp
status: proposed
iteration: 1
summary: "Upgrade cclab-nova to 95% maturity as a high-performance PydanticAI/LangGraph alternative."
history:
  - timestamp: 2026-01-28T08:39:36.751213+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-28T08:43:16.161720+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_proposal"
    action: "created"
    duration_secs: 329.39
  - timestamp: 2026-01-28T08:44:17.831235+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 61.67
  - timestamp: 2026-01-28T08:49:46.516301+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 328.68
  - timestamp: 2026-01-28T08:51:38.641844+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 112.12
impact:
  scope: major
  affected_files: 50
  new_files: 0
affected_specs:
  - id: cclab-nova-core
    path: specs/cclab-nova-core.md
    depends: []
  - id: cclab-nova-llm
    path: specs/cclab-nova-llm.md
    depends: []
  - id: cclab-nova-tools
    path: specs/cclab-nova-tools.md
    depends: []
  - id: cclab-nova-graph
    path: specs/cclab-nova-graph.md
    depends: []---

<proposal>

# Change: improve-nova-maturity

## Summary

Upgrade cclab-nova to 95% maturity as a high-performance PydanticAI/LangGraph alternative.

## Why

Current cclab-nova implementation (20% maturity) lacks critical features for enterprise-grade agent development. It misses Python bindings for core components, a robust workflow engine, structured output validation, and conversation persistence. These gaps prevent it from being a viable alternative to frameworks like PydanticAI or LangGraph. Upgrading it will enable high-performance, Rust-powered agent execution with a seamless Python developer experience.

## What Changes

- Python Bindings (PyO3) for Agent/Tools/State/Graph
- New Graph/Workflow Engine (DAG executor) in cclab-nova-graph
- Structured Output validation using cclab-shield
- RunContext-style Dependency Injection
- Conversation Persistence (Postgres/Redis adapters)
- Claude Provider fixes and streaming support
- Gateway Support (LiteLLM/OpenRouter)
- Full end-to-end Streaming Support
- Standard Toolset (Web Search, Calculator, Python REPL)
- Multi-Agent Orchestration (Supervisor/Worker patterns)

## Impact

- **Scope**: major
- **Affected Files**: ~50
- **New Files**: ~0
- Affected specs:
  - `cclab-nova-core` (no dependencies)
  - `cclab-nova-llm` (no dependencies)
  - `cclab-nova-tools` (no dependencies)
  - `cclab-nova-graph` (no dependencies)
- Affected code: `crates/cclab-nova-core`, `crates/cclab-nova-llm`, `crates/cclab-nova-tools`, `crates/cclab-nucleus/src/agent`
- **Breaking Changes**: Major API overhaul for maturity, including changes to Agent trait, Context management, and Tool execution. Integration of cclab-shield for structured output.

</proposal>
