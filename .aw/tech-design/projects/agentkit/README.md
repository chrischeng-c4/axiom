# agentkit

LLM Agent Framework — multi-crate workspace under `projects/agentkit/`.

## Workspace layout

Per `specs/workspace-slot-layout.md` (#2028) — each slot owns one part of the surface, kept disjoint so subsequent epics land cleanly.

| Crate | Path | Owner | Role |
|-------|------|-------|------|
| `agent` | `core/` | merged (#2027) | Unified inner core — `Step<I, O, D>` runtime, message/event types, error hierarchy |
| `agent-derive` | `derive/` | merged (#1952) | `#[derive(AgentTool)]` + `#[derive(AgentOutput)]` proc macros |
| `agentkit-graph` | `graph/` | Epic 3 (#2038–#2044) | Stateful graph runtime — Node/Edge, checkpoint, HITL, parallel |
| `agentkit-llm` | `llm/` | Epic 4 (#2045–#2050) | LLM primitives — PromptTemplate, OutputParser, Retriever, Embeddings, VectorStore |
| `agentkit-tools` | `tools/` | Epic 4–8 child issues | Reusable tool implementations |
| `agentkit-frontend` | `frontend/` | Epic 5 (#2051–#2053) | jet/wasm renderer hooks for run visualizer + graph debugger |
| `agentkit-backend` | `backend/` | Epic 6 (#2054–#2056) | Backend bridge surface for the mamba binding |
| `agentkit-mamba` | `mamba-binding/` | merged | Mamba binding — Python-callable Agent/Graph API |

New crates currently expose only `CODEGEN-BEGIN`/`CODEGEN-END` markers in `src/lib.rs`; the contract specs land via #2029 and are filled by their owning epic.

## Specs

<!-- type: doc lang: markdown -->

### Workspace + core (Epic 1)

| Spec | Format | Scope |
|------|--------|-------|
| [workspace-slot-layout](./specs/workspace-slot-layout.md) | Mermaid Plus class + flowchart | 8-crate slot layout, migration table, build gate (#2028) |
| [unified-inner-core](./specs/unified-inner-core.md) | Mermaid Plus class + state | `Step<I, O, D>` — Agent ⊆ Graph ⊆ Chain convergence (#2027) |

### Pre-restructure core surface

Legacy specs from `crate: agent` before the #2028 slot layout. They will be migrated under the appropriate slot directory as Epics 2–8 land:

| Spec | Format | Scope |
|------|--------|-------|
| [architecture](./logic/architecture.md) | Mermaid (C4, dependency, module) | System structure |
| [core-types](./interfaces/core/types.md) | JSON Schema | Message, Role, ToolCall, ToolResult, TokenUsage |
| [error-handling](./interfaces/error/handling.md) | JSON Schema + Mermaid flowchart | NovaError, classification |
| [agents](./logic/agents/runtime.md) | Mermaid (sequence, state, class) | Agent trait, CodingAgent, AnalystAgent |
| [llm-providers](./interfaces/llm/providers.md) | JSON Schema + Mermaid class | LLMProvider, Claude/OpenAI/Gemini |
| [tools](./tools/core.md) | JSON Schema + Mermaid class | Tool trait, Registry, Executor |
| [tools-coding](./tools/coding.md) | JSON Schema | Bash, ReadFile, WriteFile, EditFile, Glob, Grep |
| [tools-analysis](./tools/analysis.md) | JSON Schema | AskUser, TakeNote, RecordFinding, Web*, PostComment |
| [storage](./interfaces/storage/storage.md) | JSON Schema + Mermaid state | Storage trait, SessionState, backends |
| [integrations](./interfaces/platform/integrations.md) | OpenAPI 3.1 | GitHub, GitLab, Jira |
| [security](./interfaces/security/policy.md) | JSON Schema + Mermaid flowchart | SecurityPolicy, approval system |
| [streaming](./interfaces/stream/events.md) | JSON Schema + Mermaid state | StreamEvent, StreamHandler |
| [context](./logic/context.md) | JSON Schema + Mermaid flowchart | ContextManager, compression |
