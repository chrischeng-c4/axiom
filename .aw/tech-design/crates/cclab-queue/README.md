# cclab-nova Specifications

High-performance LLM agent framework with deep Rust integration.

## Competitors

- **PydanticAI**: Type-safe Python agent framework
- **LangChain**: Popular LLM application framework
- **LangGraph**: Stateful multi-agent workflow framework

## Key Differentiators

- Zero Python Byte Handling (all processing in Rust)
- GIL-free execution for maximum performance
- Deep integration with cclab ecosystem
- Type-safe Rust core with Pythonic API

## Crates & Components

| Component | Description | Spec | Status |
|-----------|-------------|------|--------|
| **Core** | Agent abstractions, context, executor | [core.md](./core.md) | Implemented |
| **LLM** | LLM provider interface | [llm.md](./llm.md) | Partial |
| **Tools** | Tool registry and execution | [tools.md](./tools.md) | Implemented |
| **Analyst Agent** | Requirements gathering and research agent | [analyst-agent.md](./analyst-agent.md) | Implemented |
| **Analysis Tools** | Specialized tools for analysis (AskUser, WebFetch) | [analysis-tools.md](./analysis-tools.md) | Implemented |
| **Integrations** | External platform connectors (GitHub, Jira) | [platform-integrations.md](./platform-integrations.md) | Implemented |
| **Storage** | Persistent session storage backends | [storage-backend.md](./storage-backend.md) | Implemented |

## Shared Artifacts

- **Schemas**: `shared/schemas/*.schema.json`
  - `core.schema.json` - Core types (Message, AgentConfig, etc.)
  - `llm.schema.json` - LLM types (CompletionRequest, etc.)
  - `tools.schema.json` - Tool types (ToolParameter, etc.)

- **APIs**: `shared/*.openrpc.json`
  - `llm-provider.openrpc.json` - LLMProvider trait
  - `tool.openrpc.json` - Tool trait and ToolExecutor

## Architecture

```
cclab-nova (Main Crate)
    │
    ├── agents/
    │   ├── Agent trait
    │   ├── CodingAgent (src/agents/coding.rs)
    │   └── AnalystAgent (src/agents/analyst.rs)
    │
    ├── tools/
    │   ├── Tool trait
    │   ├── ToolRegistry (global)
    │   ├── Builtin tools (WebSearch, Calculator)
    │   └── Analysis tools (AskUser, TakeNote, WebFetch)
    │
    ├── integrations/
    │   ├── PlatformIntegration trait
    │   ├── GitHubIntegration
    │   ├── GitLabIntegration
    │   └── JiraIntegration
    │
    └── storage/
        ├── Storage trait
        ├── MemoryStorage
        └── FileStorage
```
