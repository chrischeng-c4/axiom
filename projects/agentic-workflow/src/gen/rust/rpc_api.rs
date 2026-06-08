// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/rpc_api_types.md#schema
// CODEGEN-BEGIN
/// Output from RPC-API code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/rpc_api_types.md#schema
#[derive(Debug, Clone)]
pub struct RpcApiGenOutput {
    /// The generated async fn signatures with SPEC-REF body markers.
    pub code: String,
    /// SPEC-REF entries emitted.
    pub spec_refs: Vec<String>,
}
// CODEGEN-END
