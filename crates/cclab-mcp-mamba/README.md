# Cclab Mcp Mamba

## Brief

Cclab MCP Mamba is the Mamba native binding for MCP server primitives.

It registers the `cclab.mcp` module through the shared Mamba registry and
exposes native-call entrypoints for server creation, tool metadata registration,
tool counts, server-name lookup, a minimal stdio initialize response stub, and
streamable HTTP app handles. Full bidirectional MCP stdio dispatch is outside
the current binding surface.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Mamba MCP Server Binding | - | implemented | passing | conformance | not_ready | exposes MCP server handles, tool metadata registration, stdio initialize stub, and streamable HTTP app handles |

### Mamba MCP Server Binding

ID: mamba-mcp-server-binding
Type: DeveloperTool
Surfaces: Mamba module: `cclab.mcp`; Native ABI: `mb_mcp_server_new`, `mb_mcp_server_register_tool`, `mb_mcp_server_tool_count`, `mb_mcp_server_run_stdio`, `mb_mcp_server_streamable_http_app`, `mb_mcp_server_name`; Rust module registrar: `McpMambaModule`
EC Dimensions: behavior: `cargo test -p cclab-mcp-mamba`
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab MCP Mamba exposes MCP server primitives to Mamba scripts through the `cclab.mcp` native module, including server handles, tool registration metadata, tool counts, server-name access, a minimal stdio initialize response stub, and streamable HTTP app handles for mounting.
Gate Inventory: `cargo test -p cclab-mcp-mamba`; crates/cclab-mcp-mamba/src/lib.rs; crates/cclab-mcp-mamba/src/methods.rs; crates/cclab-mcp-mamba/src/types.rs; crates/cclab-mcp-mamba/tests/methods_test.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Mamba MCP server ABI contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-mcp-mamba`; crates/cclab-mcp-mamba/src/lib.rs; crates/cclab-mcp-mamba/src/methods.rs; crates/cclab-mcp-mamba/src/types.rs; crates/cclab-mcp-mamba/tests/methods_test.rs |
