// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/specs/mod.md#source
// CODEGEN-BEGIN
//! API Specification Generation
//!
//! Provides functions for generating various API specification formats.

pub mod asyncapi;
pub mod openapi;
pub mod openrpc;
pub mod serverless;

pub use asyncapi::{generate_asyncapi, AsyncApiInput};
pub use openapi::{generate_openapi, OpenApiInput};
pub use openrpc::{generate_openrpc, OpenRpcInput};
pub use serverless::{generate_serverless_workflow, ServerlessWorkflowInput};

// CODEGEN-END
