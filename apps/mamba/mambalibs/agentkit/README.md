# Agentkit

## Brief

Rust-native agent framework for Mamba and AW-adjacent agent workflows.

Agentkit provides the core agent runtime, structured schema contracts, typed
graph execution, Mamba runtime bindings, and MCP tool integration used to build
agentic workflows without making every consumer depend on a CLI surface.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Agent Runtime Core | - | Rust agent runtime facade for provider, tool, event, storage, integration, and protocol contracts |
| Structured Schema And Derive | - | Schema builder, validator, and `#[derive(AgentSchema)]` typed I/O contract |
| Stateful Graph Runtime | - | Typed graph runtime with event stream and checkpointing surface |
| Mamba Agent Binding | - | `mambalibs.agent` runtime module plus `cclab.agent` compatibility alias |
| MCP Tool Integration | - | MCP JSON-RPC client/server and reusable tool integration crate |

### Agent Runtime Core

Agentkit provides a Rust-native agent runtime facade for building LLM agents
with provider abstraction, tool execution, structured events, storage,
integrations, protocols, and sync adapters.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `agent::Agent`, `AgentBuilder`, `ToolSpec`,
  `LLMProvider`, `EventBus`
- Gate — behavior: `cargo test -p agent --test agent_events` - typed event
  stream and failure behavior
- Gate: `cargo test -p agent --test agent_events`
- Evidence: `cargo test -p agent --test agent_events`

### Structured Schema And Derive

Agentkit lets agent authors define structured input and output contracts
through a runtime schema builder and `#[derive(AgentSchema)]` so tool and model
payloads can be validated before execution.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `agent::Schema`; proc macro: `#[derive(AgentSchema)]`
- Gate — behavior: `cargo test -p agent-derive --test derive_smoke` - schema
  generation and validation behavior
- Gate: `cargo test -p agent-derive --test derive_smoke`
- Evidence: `cargo test -p agent-derive --test derive_smoke`

### Stateful Graph Runtime

Agentkit provides a typed graph runtime for composing stateful agent workflows
with node execution, conditional edges, checkpoints, and structured graph
events.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `agentkit_graph::Graph`, `Node`, `Checkpoint`,
  `GraphEventBus`
- Gate — behavior: `cargo test -p agentkit-graph --test graph_events` - graph
  event contract
- Gate: `cargo test -p agentkit-graph --test graph_events`
- Evidence: `cargo test -p agentkit-graph --test graph_events`

### Mamba Agent Binding

Agentkit exposes the agent runtime to Mamba code through the primary
`mambalibs.agent` module while keeping the legacy `cclab.agent` alias available
for compatibility.

- Root WI: none; this capability predates the tracker.
- Surfaces: Mamba API: `mambalibs.agent`; compatibility alias: `cclab.agent`
- Gate — behavior: `cargo test -p agentkit-binding --test registry_test` -
  namespace registration
- Gate — behavior: `cargo test -p agentkit-binding --test methods_test` -
  provider/schema/tool methods
- Gate: `cargo test -p agentkit-binding --test registry_test`
- Gate: `cargo test -p agentkit-binding --test methods_test`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Mamba namespace registration | epic | - | `cargo test -p agentkit-binding --test registry_test`; `cargo test -p agentkit-binding --test methods_test` |
| Mamba exported method contract | epic | - | `cargo test -p agentkit-binding --test methods_test` |

### MCP Tool Integration

Agentkit includes reusable MCP client and server tooling so agents can expose
and call JSON-RPC tools through a tested tool catalog and dispatch contract.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `agentkit_tools::McpClient`, `McpServer`, and reusable
  tool handlers
- Gate — behavior: `cargo test -p agentkit-tools --lib` - MCP JSON-RPC
  client/server/tool dispatch contract
- Gate: `cargo test -p agentkit-tools --lib`
- Evidence: `cargo test -p agentkit-tools --lib`
