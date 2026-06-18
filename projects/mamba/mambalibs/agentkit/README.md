# Agentkit

## Brief

Rust-native agent framework for Mamba and AW-adjacent agent workflows.

Agentkit provides the core agent runtime, structured schema contracts, typed
graph execution, Mamba runtime bindings, and MCP tool integration used to build
agentic workflows without making every consumer depend on a CLI surface.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Agent Runtime Core | - | partial | passing | smoke | not_ready | Rust agent runtime facade for provider, tool, event, storage, integration, and protocol contracts |
| Structured Schema And Derive | - | implemented | passing | conformance | not_ready | Schema builder, validator, and `#[derive(AgentSchema)]` typed I/O contract |
| Stateful Graph Runtime | - | partial | passing | smoke | not_ready | Typed graph runtime with event stream and checkpointing surface |
| Mamba Agent Binding | - | implemented | passing | smoke | not_ready | `mambalibs.agent` runtime module plus `cclab.agent` compatibility alias |
| MCP Tool Integration | - | implemented | passing | smoke | not_ready | MCP JSON-RPC client/server and reusable tool integration crate |

### Agent Runtime Core

ID: agent-runtime-core
Type: AgentFirst
Surfaces: Rust API: `agent::Agent`, `AgentBuilder`, `ToolSpec`, `LLMProvider`, `EventBus`
EC Dimensions: behavior: `cargo test -p agent --test agent_events` - typed event stream and failure behavior
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Agentkit provides a Rust-native agent runtime facade for building LLM agents with provider abstraction, tool execution, structured events, storage, integrations, protocols, and sync adapters.
Gate Inventory: `cargo test -p agent --test agent_events`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Agent event stream contract | epic | - | partial | passing | smoke | `cargo test -p agent --test agent_events` |

### Structured Schema And Derive

ID: structured-schema-and-derive
Type: DeveloperTool
Surfaces: Rust API: `agent::Schema`; proc macro: `#[derive(AgentSchema)]`
EC Dimensions: behavior: `cargo test -p agent-derive --test derive_smoke` - schema generation and validation behavior
Root WI: -
Status: confirmed
Required Verification: smoke, conformance
Promise:
Agentkit lets agent authors define structured input and output contracts through a runtime schema builder and `#[derive(AgentSchema)]` so tool and model payloads can be validated before execution.
Gate Inventory: `cargo test -p agent-derive --test derive_smoke`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Schema derive and validation contract | epic | - | implemented | passing | conformance | `cargo test -p agent-derive --test derive_smoke` |

### Stateful Graph Runtime

ID: stateful-graph-runtime
Type: RuntimeTool
Surfaces: Rust API: `agentkit_graph::Graph`, `Node`, `Checkpoint`, `GraphEventBus`
EC Dimensions: behavior: `cargo test -p agentkit-graph --test graph_events` - graph event contract
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Agentkit provides a typed graph runtime for composing stateful agent workflows with node execution, conditional edges, checkpoints, and structured graph events.
Gate Inventory: `cargo test -p agentkit-graph --test graph_events`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Graph event and failure contract | epic | - | partial | passing | smoke | `cargo test -p agentkit-graph --test graph_events` |

### Mamba Agent Binding

ID: mamba-agent-binding
Type: RuntimeTool
Surfaces: Mamba API: `mambalibs.agent`; compatibility alias: `cclab.agent`
EC Dimensions: behavior: `cargo test -p agentkit-binding --test registry_test` - namespace registration; behavior: `cargo test -p agentkit-binding --test methods_test` - provider/schema/tool methods
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Agentkit exposes the agent runtime to Mamba code through the primary `mambalibs.agent` module while keeping the legacy `cclab.agent` alias available for compatibility.
Gate Inventory: `cargo test -p agentkit-binding --test registry_test`; `cargo test -p agentkit-binding --test methods_test`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Mamba namespace registration | epic | - | implemented | passing | smoke | `cargo test -p agentkit-binding --test registry_test`; `cargo test -p agentkit-binding --test methods_test` |
| Mamba exported method contract | epic | - | implemented | passing | smoke | `cargo test -p agentkit-binding --test methods_test` |

### MCP Tool Integration

ID: mcp-tool-integration
Type: DeveloperTool
Surfaces: Rust API: `agentkit_tools::McpClient`, `McpServer`, and reusable tool handlers
EC Dimensions: behavior: `cargo test -p agentkit-tools --lib` - MCP JSON-RPC client/server/tool dispatch contract
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Agentkit includes reusable MCP client and server tooling so agents can expose and call JSON-RPC tools through a tested tool catalog and dispatch contract.
Gate Inventory: `cargo test -p agentkit-tools --lib`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| MCP client/server dispatch contract | epic | - | implemented | passing | smoke | `cargo test -p agentkit-tools --lib` |
