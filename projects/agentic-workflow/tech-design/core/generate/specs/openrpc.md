---
id: sdd-specs-openrpc-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# OpenRPC Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/specs/openrpc.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `OpenRpcInfo` | projects/agentic-workflow/src/generate/specs/openrpc.rs | struct | pub | 15 |  |
| `OpenRpcInput` | projects/agentic-workflow/src/generate/specs/openrpc.rs | struct | pub | 73 |  |
| `RpcError` | projects/agentic-workflow/src/generate/specs/openrpc.rs | struct | pub | 47 |  |
| `RpcMethod` | projects/agentic-workflow/src/generate/specs/openrpc.rs | struct | pub | 56 |  |
| `RpcParam` | projects/agentic-workflow/src/generate/specs/openrpc.rs | struct | pub | 24 |  |
| `RpcResult` | projects/agentic-workflow/src/generate/specs/openrpc.rs | struct | pub | 36 |  |
| `generate_openrpc` | projects/agentic-workflow/src/generate/specs/openrpc.rs | function | pub | 81 | generate_openrpc(input: &OpenRpcInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  OpenRpcInfo:
    type: object
    required: [title, version]
    properties:
      title: { type: string }
      version: { type: string }
      description: { type: string }
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RpcParam:
    type: object
    required: [name, required]
    properties:
      name: { type: string }
      description: { type: string }
      required:
        type: boolean
        x-serde-default: true
      schema:
        type: object
        x-rust-type: "Option<Value>"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RpcResult:
    type: object
    properties:
      name: { type: string }
      description: { type: string }
      schema:
        type: object
        x-rust-type: "Option<Value>"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RpcError:
    type: object
    required: [code, message]
    properties:
      code:
        type: integer
        x-rust-type: i32
      message: { type: string }
      data:
        type: object
        x-rust-type: "Option<Value>"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RpcMethod:
    type: object
    required: [name, summary, tags, params, errors]
    properties:
      name: { type: string }
      summary: { type: string }
      description: { type: string }
      tags:
        type: array
        items: { type: string }
        x-serde-default: true
      params:
        type: array
        items: { $ref: "#/definitions/RpcParam" }
        x-serde-default: true
      result:
        $ref: "#/definitions/RpcResult"
        x-rust-type: "Option<RpcResult>"
        x-serde-default: true
      errors:
        type: array
        items: { $ref: "#/definitions/RpcError" }
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  OpenRpcInput:
    type: object
    required: [info, methods, schemas]
    properties:
      info: { $ref: "#/definitions/OpenRpcInfo" }
      methods:
        type: array
        items: { $ref: "#/definitions/RpcMethod" }
      schemas:
        type: object
        x-rust-type: "HashMap<String, Value>"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/specs/openrpc.rs -->
```rust
//! OpenRPC 1.3 Specification Generation
//!
//! Generates OpenRPC 1.3 specifications for JSON-RPC and MCP tools.

use crate::generate::{GenerateError, Result};
use serde_json::{json, Value};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openrpc.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRpcInfo {
    pub title: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openrpc.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcParam {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub schema: Option<Value>,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openrpc.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResult {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub schema: Option<Value>,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openrpc.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openrpc.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMethod {
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub params: Vec<RpcParam>,
    #[serde(default)]
    pub result: Option<RpcResult>,
    #[serde(default)]
    pub errors: Vec<RpcError>,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openrpc.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRpcInput {
    pub info: OpenRpcInfo,
    pub methods: Vec<RpcMethod>,
    #[serde(default)]
    pub schemas: HashMap<String, Value>,
}
/// Generate an OpenRPC 1.3 specification
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openrpc.md#source
pub fn generate_openrpc(input: &OpenRpcInput) -> Result<String> {
    if input.methods.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one method required".to_string(),
        ));
    }

    let mut openrpc = json!({
        "openrpc": "1.3.2",
        "info": {
            "title": input.info.title,
            "version": input.info.version
        }
    });

    if let Some(ref desc) = input.info.description {
        openrpc["info"]["description"] = json!(desc);
    }

    // Methods
    let methods: Vec<Value> = input
        .methods
        .iter()
        .map(|m| {
            let mut method_obj = json!({
                "name": m.name,
                "summary": m.summary
            });

            if let Some(ref desc) = m.description {
                method_obj["description"] = json!(desc);
            }
            if !m.tags.is_empty() {
                method_obj["tags"] = json!(m
                    .tags
                    .iter()
                    .map(|t| json!({"name": t}))
                    .collect::<Vec<_>>());
            }

            // Params
            if !m.params.is_empty() {
                let params: Vec<Value> = m
                    .params
                    .iter()
                    .map(|p| {
                        let mut param_obj = json!({
                            "name": p.name,
                            "required": p.required
                        });
                        if let Some(ref desc) = p.description {
                            param_obj["description"] = json!(desc);
                        }
                        if let Some(ref schema) = p.schema {
                            param_obj["schema"] = schema.clone();
                        }
                        param_obj
                    })
                    .collect();
                method_obj["params"] = json!(params);
            } else {
                method_obj["params"] = json!([]);
            }

            // Result
            if let Some(ref result) = m.result {
                let mut result_obj = json!({});
                if let Some(ref name) = result.name {
                    result_obj["name"] = json!(name);
                }
                if let Some(ref desc) = result.description {
                    result_obj["description"] = json!(desc);
                }
                if let Some(ref schema) = result.schema {
                    result_obj["schema"] = schema.clone();
                }
                method_obj["result"] = result_obj;
            }

            // Errors
            if !m.errors.is_empty() {
                let errors: Vec<Value> = m
                    .errors
                    .iter()
                    .map(|e| {
                        let mut err_obj = json!({
                            "code": e.code,
                            "message": e.message
                        });
                        if let Some(ref data) = e.data {
                            err_obj["data"] = data.clone();
                        }
                        err_obj
                    })
                    .collect();
                method_obj["errors"] = json!(errors);
            }

            method_obj
        })
        .collect();

    openrpc["methods"] = json!(methods);

    // Components/schemas
    if !input.schemas.is_empty() {
        openrpc["components"] = json!({ "schemas": input.schemas });
    }

    serde_json::to_string_pretty(&openrpc).map_err(|e| GenerateError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_openrpc() {
        let input = OpenRpcInput {
            info: OpenRpcInfo {
                title: "Calculator".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Calculator API".to_string()),
            },
            methods: vec![RpcMethod {
                name: "add".to_string(),
                summary: "Add two numbers".to_string(),
                description: None,
                tags: vec![],
                params: vec![
                    RpcParam {
                        name: "a".to_string(),
                        description: None,
                        required: true,
                        schema: Some(json!({"type": "number"})),
                    },
                    RpcParam {
                        name: "b".to_string(),
                        description: None,
                        required: true,
                        schema: Some(json!({"type": "number"})),
                    },
                ],
                result: Some(RpcResult {
                    name: Some("result".to_string()),
                    description: None,
                    schema: Some(json!({"type": "number"})),
                }),
                errors: vec![],
            }],
            schemas: HashMap::new(),
        };

        let result = generate_openrpc(&input).unwrap();
        assert!(result.contains("\"openrpc\": \"1.3.2\""));
        assert!(result.contains("Calculator"));
        assert!(result.contains("add"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/specs/openrpc.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete OpenRPC specification generation module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
