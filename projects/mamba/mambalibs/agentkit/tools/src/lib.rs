//! agentkit-tools — reusable agent tool implementations and external
//! tool-source integrations.
//!
//! Currently houses:
//!
//! | Module        | Issue  | Purpose                                  |
//! |---------------|--------|------------------------------------------|
//! | `mcp_client`  | #2063  | Consume external MCP servers as tools    |
//! | `mcp_server`  | #2064  | Publish agentkit tools as an MCP server  |

pub mod mcp_client;
pub mod mcp_server;

pub use mcp_client::{
    McpClient, McpTool, McpToolCallResult, McpTransport, JSONRPC_VERSION, MCP_PROTOCOL_VERSION,
};
pub use mcp_server::{
    FnHandler, McpHandlerFuture, McpServer, McpToolHandler, ERR_INTERNAL, ERR_INVALID_PARAMS,
    ERR_METHOD_NOT_FOUND,
};
