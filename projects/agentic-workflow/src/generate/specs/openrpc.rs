// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/specs/openrpc.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
