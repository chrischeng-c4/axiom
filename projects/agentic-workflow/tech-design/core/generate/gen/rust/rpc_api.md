---
id: sdd-generate-gen-rust-rpc-api
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# RPC API Structural Generator

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/rust/rpc_api.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `RpcApiGenOutput` | projects/agentic-workflow/src/generate/gen/rust/rpc_api.rs | struct | pub | 10 |  |
| `generate_rpc_api` | projects/agentic-workflow/src/generate/gen/rust/rpc_api.rs | function | pub | 23 | generate_rpc_api(     openrpc_yaml: &Value,     spec_path: &str,     config: &RustConfig, ) -> RpcApiGenOutput |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/rust/rpc_api.rs -->
```rust
//! RPC-API structural generator.
//!
//! Produces async fn signatures from OpenRPC 1.3 YAML frontmatter.
//! ~90% coverage — method bodies get SPEC-REF markers.

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R3

use crate::generate::marker::{emit_spec_ref, Lang};
use crate::generate::types::RustConfig;
use serde_yaml::Value;

/// Output from RPC-API code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/rpc_api_types.md#schema
#[derive(Debug, Clone)]
pub struct RpcApiGenOutput {
    /// The generated async fn signatures with SPEC-REF body markers.
    pub code: String,
    /// SPEC-REF entries emitted.
    pub spec_refs: Vec<String>,
}

/// Generate Rust async fn signatures from OpenRPC YAML.
///
/// Each OpenRPC method becomes an `async fn` with typed parameters and
/// a SPEC-REF body marker (90% coverage — body logic not generated).
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R3
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R6
pub fn generate_rpc_api(
    openrpc_yaml: &Value,
    spec_path: &str,
    config: &RustConfig,
) -> RpcApiGenOutput {
    let config = config.merge_overrides(openrpc_yaml);
    let vis = config.vis_prefix();

    let methods = openrpc_yaml
        .get("methods")
        .and_then(|v| v.as_sequence())
        .cloned()
        .unwrap_or_default();

    let mut spec_refs = Vec::new();
    let mut lines = Vec::new();

    for method in &methods {
        let method_name = method
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("method")
            .replace('-', "_")
            .replace('.', "_");

        let summary = method.get("summary").and_then(|v| v.as_str()).unwrap_or("");

        // Doc comment
        if !summary.is_empty() {
            lines.push(format!("/// {}", summary));
        }

        // Build parameter list
        let params: Vec<String> = method
            .get("params")
            .and_then(|v| v.as_sequence())
            .map(|ps| {
                ps.iter()
                    .filter_map(|p| {
                        let pname = p.get("name")?.as_str()?.replace('-', "_");
                        let ptype = p
                            .get("schema")
                            .and_then(|s| json_schema_to_rust_type(s))
                            .unwrap_or_else(|| "serde_json::Value".to_string());
                        Some(format!("{}: {}", pname, ptype))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Result type
        let result_type = method
            .get("result")
            .and_then(|r| r.get("schema"))
            .and_then(|s| json_schema_to_rust_type(s))
            .unwrap_or_else(|| "serde_json::Value".to_string());

        // Signature
        let param_str = if params.is_empty() {
            "&self".to_string()
        } else {
            format!("&self, {}", params.join(", "))
        };

        lines.push(format!(
            "{}async fn {}({}) -> Result<{}> {{",
            vis, method_name, param_str, result_type
        ));

        // SPEC-REF body marker
        let section_id = method_name.replace('_', "-");
        let marker = emit_spec_ref(
            spec_path,
            &section_id,
            &format!("Implement RPC method {}", method_name),
            Lang::Rust,
        );
        for marker_line in marker.lines() {
            lines.push(format!("    {}", marker_line));
        }
        spec_refs.push(format!("{}#{}", spec_path, section_id));

        lines.push("    todo!()".to_string());
        lines.push("}".to_string());
        lines.push(String::new());
    }

    RpcApiGenOutput {
        code: lines.join("\n").trim_end().to_string(),
        spec_refs,
    }
}

fn json_schema_to_rust_type(schema: &Value) -> Option<String> {
    let type_str = schema.get("type")?.as_str()?;
    Some(match type_str {
        "string" => "String".to_string(),
        "integer" => "i64".to_string(),
        "number" => "f64".to_string(),
        "boolean" => "bool".to_string(),
        "array" => {
            let item = schema
                .get("items")
                .and_then(|i| json_schema_to_rust_type(i))
                .unwrap_or_else(|| "serde_json::Value".to_string());
            format!("Vec<{}>", item)
        }
        "object" => "serde_json::Value".to_string(),
        _ => "serde_json::Value".to_string(),
    })
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/rpc_api.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete canonical RPC API structural generator
      module.
```

# Reviews

