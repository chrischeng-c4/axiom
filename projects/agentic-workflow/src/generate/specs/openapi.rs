// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#source
// CODEGEN-BEGIN
//! OpenAPI 3.1 Specification Generation
//!
//! Generates OpenAPI 3.1 specifications for HTTP REST APIs.

use crate::generate::{GenerateError, Result};
use serde_json::{json, Value};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// HTTP method.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// Parameter location.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterIn {
    Path,
    Query,
    Header,
    Cookie,
}

/// API info.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInfo {
    /// API title.
    pub title: String,
    /// API version.
    pub version: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Server definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// Server URL.
    pub url: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
}

/// API parameter.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name.
    pub name: String,
    /// Parameter location.
    #[serde(rename = "in")]
    pub location: ParameterIn,
    /// Whether parameter is required.
    #[serde(default)]
    pub required: Option<bool>,
    /// Parameter description.
    #[serde(default)]
    pub description: Option<String>,
    /// JSON Schema for the parameter.
    #[serde(default)]
    pub schema: Option<Value>,
}

/// Request body.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    /// Body description.
    #[serde(default)]
    pub description: Option<String>,
    /// Whether required.
    #[serde(default)]
    pub required: Option<bool>,
    /// Content type.
    #[serde(default)]
    pub content_type: Option<String>,
    /// JSON Schema.
    #[serde(default)]
    pub schema: Option<Value>,
}

/// API response.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// HTTP status code.
    pub status: String,
    /// Response description.
    pub description: String,
    /// Content type.
    #[serde(default)]
    pub content_type: Option<String>,
    /// JSON Schema.
    #[serde(default)]
    pub schema: Option<Value>,
}

/// Path operation.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathOperation {
    /// URL path template.
    pub path: String,
    /// HTTP method.
    pub method: HttpMethod,
    /// Operation summary.
    pub summary: String,
    /// Operation description.
    #[serde(default)]
    pub description: Option<String>,
    /// Operation identifier.
    #[serde(default)]
    pub operation_id: Option<String>,
    /// Operation tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Operation parameters.
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    /// Request body.
    #[serde(default)]
    pub request_body: Option<RequestBody>,
    /// Operation responses.
    #[serde(default)]
    pub responses: Vec<ApiResponse>,
}

/// Input for OpenAPI generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiInput {
    /// API info.
    pub info: ApiInfo,
    /// Path operations.
    pub paths: Vec<PathOperation>,
    /// Server definitions.
    #[serde(default)]
    pub servers: Vec<Server>,
    /// Component schemas keyed by name.
    #[serde(default)]
    pub schemas: HashMap<String, Value>,
}
/// Generate an OpenAPI 3.1 specification
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#source
pub fn generate_openapi(input: &OpenApiInput) -> Result<String> {
    if input.paths.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one path required".to_string(),
        ));
    }

    // Build info object
    let mut info_obj = json!({
        "title": input.info.title,
        "version": input.info.version
    });
    if let Some(ref desc) = input.info.description {
        info_obj["description"] = json!(desc);
    }

    // Build paths object
    let mut paths_obj = serde_json::Map::new();
    for op in &input.paths {
        let method = match op.method {
            HttpMethod::Get => "get",
            HttpMethod::Post => "post",
            HttpMethod::Put => "put",
            HttpMethod::Patch => "patch",
            HttpMethod::Delete => "delete",
        };

        let mut operation = json!({
            "summary": op.summary
        });

        if let Some(ref desc) = op.description {
            operation["description"] = json!(desc);
        }
        if let Some(ref op_id) = op.operation_id {
            operation["operationId"] = json!(op_id);
        }
        if !op.tags.is_empty() {
            operation["tags"] = json!(op.tags);
        }

        // Parameters
        if !op.parameters.is_empty() {
            let params: Vec<Value> = op
                .parameters
                .iter()
                .map(|p| {
                    let mut param_obj = json!({
                        "name": p.name,
                        "in": match p.location {
                            ParameterIn::Path => "path",
                            ParameterIn::Query => "query",
                            ParameterIn::Header => "header",
                            ParameterIn::Cookie => "cookie",
                        }
                    });
                    if let Some(required) = p.required {
                        param_obj["required"] = json!(required);
                    }
                    if let Some(ref desc) = p.description {
                        param_obj["description"] = json!(desc);
                    }
                    if let Some(ref schema) = p.schema {
                        param_obj["schema"] = schema.clone();
                    }
                    param_obj
                })
                .collect();
            operation["parameters"] = json!(params);
        }

        // Request body
        if let Some(ref req_body) = op.request_body {
            let content_type = req_body
                .content_type
                .as_deref()
                .unwrap_or("application/json");
            let mut req_body_obj = json!({});
            if let Some(ref desc) = req_body.description {
                req_body_obj["description"] = json!(desc);
            }
            if let Some(required) = req_body.required {
                req_body_obj["required"] = json!(required);
            }
            if let Some(ref schema) = req_body.schema {
                req_body_obj["content"] = json!({
                    content_type: { "schema": schema }
                });
            }
            operation["requestBody"] = req_body_obj;
        }

        // Responses
        if !op.responses.is_empty() {
            let mut responses_obj = serde_json::Map::new();
            for resp in &op.responses {
                let content_type = resp.content_type.as_deref().unwrap_or("application/json");
                let mut resp_obj = json!({ "description": resp.description });
                if let Some(ref schema) = resp.schema {
                    resp_obj["content"] = json!({
                        content_type: { "schema": schema }
                    });
                }
                responses_obj.insert(resp.status.clone(), resp_obj);
            }
            operation["responses"] = Value::Object(responses_obj);
        } else {
            operation["responses"] = json!({ "200": { "description": "Success" } });
        }

        // Add to paths
        let path_entry = paths_obj
            .entry(op.path.clone())
            .or_insert_with(|| json!({}));
        if let Value::Object(ref mut path_map) = path_entry {
            path_map.insert(method.to_string(), operation);
        }
    }

    // Build final OpenAPI document
    let mut openapi = json!({
        "openapi": "3.1.0",
        "info": info_obj,
        "paths": Value::Object(paths_obj)
    });

    if !input.servers.is_empty() {
        openapi["servers"] = json!(input.servers);
    }

    if !input.schemas.is_empty() {
        openapi["components"] = json!({ "schemas": input.schemas });
    }

    serde_json::to_string_pretty(&openapi).map_err(|e| GenerateError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_openapi() {
        let input = OpenApiInput {
            info: ApiInfo {
                title: "User API".to_string(),
                version: "1.0.0".to_string(),
                description: Some("API for user management".to_string()),
            },
            paths: vec![PathOperation {
                path: "/users".to_string(),
                method: HttpMethod::Get,
                summary: "List users".to_string(),
                description: None,
                operation_id: Some("listUsers".to_string()),
                tags: vec!["users".to_string()],
                parameters: vec![],
                request_body: None,
                responses: vec![ApiResponse {
                    status: "200".to_string(),
                    description: "List of users".to_string(),
                    content_type: None,
                    schema: Some(json!({"type": "array"})),
                }],
            }],
            servers: vec![],
            schemas: HashMap::new(),
        };

        let result = generate_openapi(&input).unwrap();
        assert!(result.contains("\"openapi\": \"3.1.0\""));
        assert!(result.contains("User API"));
        assert!(result.contains("/users"));
        assert!(result.contains("listUsers"));
    }
}

// CODEGEN-END
