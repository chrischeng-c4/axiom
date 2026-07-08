// SPEC-MANAGED: libs/compass/tech-design/semantic/source/libs-compass-src-lsp-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! LSP Server for Argus
//!
//! Provides Language Server Protocol support for real-time code analysis.

mod server;

pub use server::{run_server, run_server_tcp, ArgusServer};
// CODEGEN-END
